use serde_derive::{Serialize, Deserialize};

use crate::constants::{ROOM_WIDTH, ROOM_LENGTH, DOORWAY_LENGTH, DOOR_WIDTH};
use crate::math::{Vec2, vec2};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub rooms: [Vec<Room>; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Room {
    FullRoom(Door), Corridor(Door), Empty
}

pub type Door = Vec<(i8, i8)>;

pub fn rooms_in_col(col: usize) -> usize {
    match col {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 4,
        5 => 3,
        6 => 2,
        7 => 1,
        _ => panic!("Column index out of bounds {}", col),
    }
}

pub fn room_corner_position(col: usize, row: usize) -> Vec2 {
    let c = col as f32;
    let r = row as f32;
    let rooms_in_column = rooms_in_col(col);

    let row_height = ROOM_LENGTH + DOORWAY_LENGTH;

    let x = c * (ROOM_WIDTH + DOORWAY_LENGTH);
    let y = row_height * (r - 0.5 * (rooms_in_column - 1) as f32)
        - ROOM_LENGTH / 2.;
    vec2(x, y)
}

pub fn doorway_bounds((col, row): (usize, usize), (dx, dy): (i8, i8)) -> (Vec2, Vec2) {
    // NOTE: note finished!
    let this_pos = room_corner_position(col, row);
    let other_pos = room_corner_position((col as i8 + dx) as usize, (row as i8 + dy) as usize);

    if dx != 0 {
        let offset = vec2(0., 0.);
        (this_pos + offset, vec2(DOORWAY_LENGTH / 2., DOOR_WIDTH))
    } else if dy != 0 {
        let offset = vec2(0., 0.);
        (this_pos + offset, vec2(DOOR_WIDTH, DOORWAY_LENGTH / 2.))
    } else {
        panic!("Can't have doorway to self");
    }
}

use Room::*;
pub fn example_level() -> Level {
    Level {
        rooms: [
            vec![FullRoom(vec![(1, 0)])],
            vec![
                FullRoom(vec![(-1, 0), (0, 1)]),
                Corridor(vec![(0, -1), (1, 0)]),
            ],
            vec![
                FullRoom(vec![(-1, 0), (1, 1)]),
                Corridor(vec![(-1, 0), (0, 1)]),
                FullRoom(vec![(0, -1), (1, 1)]),
            ],
            vec![
                FullRoom(vec![(0, 1), (1, 0)]),
                Corridor(vec![(-1, 0), (0, -1), (1, 0)]),
                Corridor(vec![(1, 0), (0, 1)]),
                Corridor(vec![(-1, 0), (0, -1), (1, 0)]),
            ],
            vec![
                FullRoom(vec![(-1, 0), (0, 1)]),
                FullRoom(vec![(-1, 0), (0, -1), (1, -1)]),
                FullRoom(vec![(-1, 0), (1, -1)]),
                Corridor(vec![(-1, 0), (0, -1)]),
            ],
            vec![
                FullRoom(vec![(-1, 1), (1, 0), (0, 1)]),
                FullRoom(vec![(-1, 1), (0, -1), (1, -1)]),
                Empty,
            ],
            vec![
                FullRoom(vec![(-1, 0), (-1, 1), (1, 0)]),
                Empty,
            ],
            vec![FullRoom(vec![(-1, 0)])]
        ]
    }
}

// pub fn generate_level() -> Level {
//     let mut rng = rand::thread_rng();

//     let mut rooms = [
//         vec![Room],
//     ];

//     Level {
//         rooms
//     }
// }
