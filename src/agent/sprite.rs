use std::path::Path;

use luminance::context::GraphicsContext;
use luminance::pipeline::TextureBinding;
use luminance::pixel::{NormRGBA8UI, NormUnsigned};
use luminance::shader::Uniform;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;

use sdl2::image::LoadSurface;

pub type Sprite = Texture<GL33, Dim2, NormRGBA8UI>;

#[derive(UniformInterface)]
pub struct SpriteInterface {
    pub tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub fn load_sprite(
    surface: &mut impl GraphicsContext<Backend = GL33>,
    path: impl AsRef<Path>,
) -> Sprite {
    let image = sdl2::surface::Surface::from_file(path).unwrap();
    let (width, height) = image.size();
    let bytes = image.without_lock().unwrap();

    let sampler = Sampler {
        mag_filter: luminance::texture::MagFilter::Nearest,
        min_filter: luminance::texture::MinFilter::Nearest,
        ..Default::default()
    };

    let mut texture =
        Texture::new(surface, [width, height], 0, sampler).expect("Failed to create texture");

    texture.upload_raw(GenMipmaps::Yes, bytes).unwrap();

    texture
}
