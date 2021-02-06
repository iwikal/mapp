use std::path::Path;

use luminance::context::GraphicsContext;
use luminance::pipeline::TextureBinding;
use luminance::pixel::{NormRGBA8UI, NormUnsigned};
use luminance::shader::Uniform;
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Texture, Wrap};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;

use sdl2::image::LoadSurface;

pub type Sprite = Texture<GL33, Dim2, NormRGBA8UI>;

#[derive(UniformInterface)]
pub struct SpriteInterface {
    pub tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    pub view: Uniform<[[f32; 4]; 4]>,
    pub projection: Uniform<[[f32; 4]; 4]>,
}

pub fn load_texture(
    surface: &mut impl GraphicsContext<Backend = GL33>,
    path: impl AsRef<Path>,
) -> Texture<GL33, Dim2, NormRGBA8UI> {
    let image = sdl2::surface::Surface::from_file(path).unwrap();
    let (width, height) = image.size();
    let bytes = image.without_lock().unwrap();

    let sampler = Sampler {
        mag_filter: MagFilter::Nearest,
        min_filter: MinFilter::Nearest,
        wrap_r: Wrap::Repeat,
        wrap_s: Wrap::Repeat,
        wrap_t: Wrap::Repeat,
        ..Default::default()
    };

    let mut texture =
        Texture::new(surface, [width, height], 4, sampler).expect("Failed to create texture");

    texture.upload_raw(GenMipmaps::Yes, bytes).unwrap();

    texture
}

pub fn load_sprite(
    surface: &mut impl GraphicsContext<Backend = GL33>,
    path: impl AsRef<Path>,
) -> Sprite {
    load_texture(surface, path)
}
