use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

pub struct Assets<'ttf, 'r> {
    pub font: sdl2::ttf::Font<'ttf, 'r>,

    pub menu_background: Texture<'r>,
    pub end_background: Texture<'r>,
    pub sounds: SoundAssets,
}

pub struct SoundAssets {
    pub achtung_blitzkrieg_engine: Chunk,
    pub el_pollo_romero_engine: Chunk,
    pub howdy_cowboy_engine: Chunk,
    pub suka_blyat_engine: Chunk,
    pub explosion: Chunk,
    pub powerup: Chunk,
    pub gun: Chunk,
    pub laser_fire_sound: Chunk,
    pub laser_charge_sound: Chunk,
}

impl<'ttf, 'r> Assets<'ttf, 'r> {
    pub fn new(
        texture_creator: &'r TextureCreator<WindowContext>,
        ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext,
        sounds: SoundAssets,
    ) -> Assets<'ttf, 'r> {
        let load_tex = |path: &str| {
            let mut tex = texture_creator
                .load_texture(path)
                .expect(&format!("Could not load {}", path));
            tex.set_blend_mode(sdl2::render::BlendMode::Blend);
            tex
        };

        Assets {
            font: ttf_context
                .load_font("resources/yoster.ttf", 15)
                .expect("Could not find font!"),
            menu_background: load_tex("resources/menu_background.png"),
            end_background: load_tex("resources/endscreen.png"),
            sounds: SoundAssets::new(),
        }
    }
}

impl SoundAssets {
    pub fn new() -> Self {
        let mut sounds = SoundAssets {
            achtung_blitzkrieg_engine: Chunk::from_file(
                "resources/audio/achtungblitzkrieg-engine.ogg",
            )
            .unwrap(),
            el_pollo_romero_engine: Chunk::from_file("resources/audio/elpolloromero-engine.ogg")
                .unwrap(),
            howdy_cowboy_engine: Chunk::from_file("resources/audio/howdycowboy-engine.ogg")
                .unwrap(),
            suka_blyat_engine: Chunk::from_file("resources/audio/sukablyat-engine.ogg").unwrap(),
            powerup: Chunk::from_file("resources/audio/powerup.ogg").unwrap(),
            explosion: Chunk::from_file("resources/audio/explosion.ogg").unwrap(),
            gun: Chunk::from_file("resources/audio/gun.ogg").unwrap(),
            laser_fire_sound: Chunk::from_file("resources/audio/laserfire.ogg").unwrap(),
            laser_charge_sound: Chunk::from_file("resources/audio/lasercharge.ogg").unwrap(),
        };

        // Volume is on a scale from 0 to 128
        sounds.achtung_blitzkrieg_engine.set_volume(30);
        sounds.el_pollo_romero_engine.set_volume(30);
        sounds.howdy_cowboy_engine.set_volume(30);
        sounds.suka_blyat_engine.set_volume(30);

        sounds
    }
}
