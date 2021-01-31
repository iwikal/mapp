use serde_derive::{Serialize, Deserialize};

use crate::constants::{ROOM_WIDTH, ROOM_LENGTH, DOORWAY_LENGTH, DOOR_WIDTH};
use crate::math::{Vec2, vec2};
use ultraviolet::Mat2;

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
        0 | 7 => 1,
        1 | 6 => 2,
        2 | 5 => 3,
        3 | 4 => 4,
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

pub fn doorway_transform((col, row): (usize, usize), (dx, dy): (i8, i8)) -> (Mat2, Vec2) {
    let delta = (dx, dy);
    match delta {
        (0, 0) => panic!("invalid doorway {:?}", delta),
        (0, -1..=1) => {
            let rotation = Mat2::identity() * dy as f32;
            let translation = vec2(0., ROOM_LENGTH / 2.) * dy as f32;
            (rotation, translation)
        }
        (-1..=1, -1..=1) => {
            let rotation = Mat2::new(
                vec2(0., -1.),
                vec2(1., 0.),
            ) * dx as f32;

            let this_room = room_corner_position(col, row);
            let other_room = room_corner_position((col as i8 + dx) as _, (row as i8 + dy) as _);
            let midpoint = (other_room - this_room) / 2.;

            let translation = vec2(ROOM_WIDTH / 2. * dx as f32, midpoint.y);

            (rotation, translation)
        }
        _ => panic!("invalid doorway {:?}", delta),
    }
}

pub fn doorway_bounds((col, row): (usize, usize), (dx, dy): (i8, i8)) -> (Vec2, Vec2) {
    let this_pos = room_corner_position(col, row);
    let room_center = this_pos + vec2(ROOM_WIDTH / 2., ROOM_LENGTH / 2.);
    let (rotation, translation) = doorway_transform((col, row), (dx, dy));
    let door_pos = room_center + translation;

    let one_corner = vec2(DOOR_WIDTH / 2., DOORWAY_LENGTH / 2.);
    let other_corner = one_corner * -1.;

    (
        rotation * one_corner + door_pos,
        rotation * other_corner + door_pos,
    )
}

use Room::*;
pub fn example_level() -> Level {
    Level {
        rooms: [
            vec![FullRoom(vec![(1, 0)])],
            vec![
                FullRoom(vec![(-1, 0), (0, 1), (1, 0)]),
                Corridor(vec![(0, -1), (1, 0)]),
            ],
            vec![
                FullRoom(vec![(-1, 0), (1, 1)]),
                Corridor(vec![(-1, 0), (0, 1)]),
                FullRoom(vec![(0, -1), (1, 1)]),
            ],
            vec![
                FullRoom(vec![(0, 1), (1, 0)]),
                Corridor(vec![(-1, -1), (0, -1), (1, 0)]),
                Corridor(vec![(1, 0), (0, 1)]),
                Corridor(vec![(-1, -1), (0, -1), (1, 0)]),
            ],
            vec![
                FullRoom(vec![(-1, 0), (0, 1)]),
                FullRoom(vec![(-1, 0), (0, -1), (1, -1)]),
                FullRoom(vec![(-1, 0), (1, -1), (0, 1)]),
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
