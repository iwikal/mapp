use sdl2::render::Canvas;
use sdl2::video::Window;

use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{self, vec2, Vec2};

use crate::assets::Assets;
use crate::rendering;

pub struct Map {
    // add client side state
}

impl Map {
    pub fn new() -> Map {
        Map {
            // init client stuff
        }
    }

    pub fn update(&mut self, delta_time: f32, game_state: &GameState, my_id: u64) {
        // update client side stuff
    }

    pub fn draw(&self, my_id: u64, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let (screen_w, screen_h) = canvas.logical_size();
        let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        // draw some stuff

        Ok(())
    }
}
