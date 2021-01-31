use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::constants::{self, DOORWAY_LENGTH, ROOM_LENGTH, ROOM_WIDTH, SCREEN_PADDING};
use libplen::gamestate::GameState;
use libplen::level::{self, Level, Room};
use libplen::math::vec2;

pub struct Map {
    pub level: Level,
}

impl Map {
    pub fn new(level: Level) -> Map {
        Map { level }
    }

    pub fn update(&mut self, _delta_time: f32, _game_state: &GameState, _my_id: u64) {
        // update client side stuff
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let (screen_w, screen_h) = canvas.logical_size();
        let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        let map_width = ROOM_WIDTH * 8. + DOORWAY_LENGTH * 7. + SCREEN_PADDING * 2.;
        let scale = screen_w as f32 / map_width;

        for col in 0..8 {
            let rooms_in_column = level::rooms_in_col(col);
            for row in 0..rooms_in_column {
                match &self.level.rooms[col][row] {
                    Room::FullRoom(doors) => {
                        let room_pos = level::room_corner_position(col, row);
                        let dest_rect = sdl2::rect::Rect::new(
                            ((room_pos.x + SCREEN_PADDING) * scale) as i32,
                            (screen_center.y + room_pos.y * scale) as i32,
                            (ROOM_WIDTH * scale) as u32,
                            (ROOM_LENGTH * scale) as u32,
                        );

                        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
                        canvas.fill_rect(dest_rect)?;

                        self.draw_doors(canvas, doors, (col, row), scale)?;
                    }
                    Room::Corridor(doors) => {
                        // TODO: Draw corridors

                        self.draw_doors(canvas, doors, (col, row), scale)?;
                    }
                    Room::Empty => {}
                }
            }
        }

        Ok(())
    }

    fn draw_doors(
        &self,
        canvas: &mut Canvas<Window>,
        doors: &[(i8, i8)],
        grid_pos: (usize, usize),
        scale: f32,
    ) -> Result<(), String> {
        let (screen_w, screen_h) = canvas.logical_size();
        let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        for door in doors {
            let (door_corner1, door_corner2) = level::doorway_bounds(grid_pos, *door);
            let door_pos = vec2(
                door_corner1.x.min(door_corner2.x),
                door_corner1.y.min(door_corner2.y),
            );
            let door_size = vec2(
                (door_corner1.x - door_corner2.x).abs(),
                (door_corner1.y - door_corner2.y).abs(),
            );
            let dest_rect = sdl2::rect::Rect::new(
                ((door_pos.x + SCREEN_PADDING) * scale) as i32,
                (screen_center.y + door_pos.y * scale) as i32,
                (door_size.x * scale) as u32,
                (door_size.y * scale) as u32,
            );

            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
            canvas.fill_rect(dest_rect)?;
        }
        Ok(())
    }
}
