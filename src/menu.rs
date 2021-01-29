use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::Assets;
use crate::rendering;
use libplen::constants;
use libplen::math::vec2;

pub struct MenuState {
    pub name: String,
    // more menu options...
}

impl MenuState {
    pub fn new() -> MenuState {
        MenuState {
            name: String::new(),
            // more menu options...
        }
    }
}

impl MenuState {
    fn draw_player_name(
        &mut self,
        canvas: &mut Canvas<Window>,
        assets: &Assets,
    ) -> Result<(), String> {
        let (nx, ny) = constants::NAME_POS;
        let text = assets
            .font
            .render(&format!("Hello {}. Press Enter to join", self.name))
            .blended((255, 255, 255))
            .expect("Could not render text");

        let texture_creator = canvas.texture_creator();
        let text_texture = texture_creator.create_texture_from_surface(text).unwrap();

        let res_offset = rendering::calculate_resolution_offset(canvas);
        rendering::draw_texture(canvas, &text_texture, vec2(nx + 10., ny + 10.) + res_offset)
    }

    pub fn update(&mut self) {
        // update menu state
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, assets: &Assets) -> Result<(), String> {
        let (width, height) = canvas.logical_size();
        canvas.set_draw_color(constants::MENU_BACKGROUND_COLOR);
        canvas.clear();

        self.draw_player_name(canvas, assets)?;

        canvas.present();
        Ok(())
    }
}
