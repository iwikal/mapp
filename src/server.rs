use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;
use std::vec;

use unicode_truncate::UnicodeTruncateStr;

use libplen::constants;
use libplen::gamestate;
use libplen::level::{self, Level};
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use libplen::player::{Player, PlayerType};

fn send_bytes(bytes: &[u8], stream: &mut TcpStream) -> io::Result<()> {
    let mut start = 0;
    loop {
        match stream.write(&bytes[start..bytes.len()]) {
            Ok(n) => {
                if n < bytes.len() - start {
                    start = start + n;
                } else {
                    break Ok(());
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                io::ErrorKind::Interrupted => continue,
                _ => return Err(e),
            },
        }
    }
}

fn send_server_message(msg: &ServerMessage, stream: &mut TcpStream) -> io::Result<()> {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    send_bytes(&length.to_be_bytes(), stream)?;
    send_bytes(&data, stream)
}

struct Client {
    id: u64,
    message_reader: MessageReader,
    input: ClientInput,
}

struct Server {
    listener: TcpListener,
    connections: Vec<Client>,
    state: gamestate::GameState,
    level: Level,
    next_id: u64,
    last_time: Instant,
}

impl Server {
    pub fn new() -> Self {
        let listener = TcpListener::bind("0.0.0.0:4444").unwrap();

        listener.set_nonblocking(true).unwrap();

        println!("Listening on 0.0.0.0:4444");

        Self {
            listener,
            connections: vec![],
            next_id: 0,
            last_time: Instant::now(),
            state: gamestate::GameState::new(),
            level: level::example_level(),
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.last_time.elapsed();
        let delta_time = constants::DELTA_TIME;
        let dt_duration = std::time::Duration::from_millis(constants::SERVER_SLEEP_DURATION);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }
        self.last_time = Instant::now();

        self.state.update(delta_time);

        self.accept_new_connections();
        self.update_clients(delta_time);
    }

    fn accept_new_connections(&mut self) {
        // Read data from clients
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    stream.set_nonblocking(true).unwrap();
                    println!("Got new connection {}", self.next_id);
                    if let Err(_) =
                        send_server_message(&ServerMessage::AssignId(self.next_id), &mut stream)
                    {
                        println!("Could not send assign id message");
                        continue;
                    }
                    self.connections.push(Client {
                        id: self.next_id,
                        message_reader: MessageReader::new(stream),
                        input: ClientInput::new(),
                    });
                    self.next_id += 1;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // wait until network socket is ready, typically implemented
                    // via platform-specific APIs such as epoll or IOCP
                    break;
                }
                e => {
                    e.expect("Socket listener error");
                }
            }
        }
    }

    fn update_clients(&mut self, delta_time: f32) {
        // Send data to clients
        let mut clients_to_delete = vec![];
        let mut sounds_to_play = vec![];

        macro_rules! remove_player_on_disconnect {
            ($op:expr, $id:expr) => {
                match $op {
                    Ok(_) => {}
                    Err(e) => match e.kind() {
                        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                            println!("Player {} disconnected", $id);
                            clients_to_delete.push($id);
                            break;
                        }
                        e => {
                            panic!("Unhandled network issue: {:?}", e)
                        }
                    },
                };
            };
        }

        let mut players_to_add = vec![];

        for client in self.connections.iter_mut() {
            remove_player_on_disconnect!(client.message_reader.fetch_bytes(), client.id);

            for message in client.message_reader.iter() {
                match bincode::deserialize(&message) {
                    Ok(ClientMessage::Input(input)) => {
                        client.input = input;
                    }
                    Ok(ClientMessage::JoinTeam {
                        team_id,
                        player_type,
                        name,
                    }) => {
                        players_to_add.push((client.id, team_id, player_type, name));
                    }
                    Ok(ClientMessage::SetName { mut name }) => {
                        if name.trim().len() != 0 {
                            name = name.trim().unicode_truncate(20).0.to_string()
                        } else {
                            name = "Mr Whitespace".into();
                        }
                        self.state.set_player_name(client.id, name);
                    }
                    Err(_) => {
                        println!("Could not decode message from {}, deleting", client.id);
                        clients_to_delete.push(client.id);
                    }
                }
            }

            if let Some(player) = self.state.get_mut_player_by_id(client.id) {
                player.update(delta_time, &client.input);
            }

            let result = send_server_message(
                &ServerMessage::GameState(self.state.clone()),
                &mut client.message_reader.stream,
            );
            remove_player_on_disconnect!(result, client.id);
        }

        for (client_id, team_id, player_type, name) in players_to_add {
            self.try_add_player_to_team(client_id, team_id, player_type, name);
        }

        for (sound, pos) in &sounds_to_play {
            for client in self.connections.iter_mut() {
                let result = send_server_message(
                    &ServerMessage::PlaySound(*sound, *pos),
                    &mut client.message_reader.stream,
                );
                remove_player_on_disconnect!(result, client.id);
            }
        }

        for id in &clients_to_delete {
            self.state.remove_player(*id);
        }

        self.connections
            .retain(|client| !clients_to_delete.contains(&client.id));
    }

    fn try_add_player_to_team(
        &mut self,
        player_id: u64,
        team_id: u64,
        player_type: PlayerType,
        name: String,
    ) {
        self.state
            .try_add_player_to_team(player_id, team_id, player_type, name);
    }
}

fn main() {
    let mut server = Server::new();
    loop {
        server.update();
    }
}
