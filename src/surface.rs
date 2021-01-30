use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::framebuffer::FramebufferError;
use luminance::texture::Dim2;
pub use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
pub use sdl2;
use std::fmt;
use std::os::raw::c_void;

/// Error that can be risen while creating a surface.
#[non_exhaustive]
#[derive(Debug)]
pub enum Sdl2SurfaceError {
    /// Initialization of the surface went wrong.
    InitError(String),
    /// Window creation failed.
    WindowCreationFailed(sdl2::video::WindowBuildError),
    /// Failed to create an OpenGL context.
    GlContextInitFailed(String),
    /// No available video mode.
    VideoInitError(String),
    /// The graphics state is not available.
    ///
    /// This error is generated when the initialization code is called on a thread on which the
    /// graphics state has already been acquired.
    GraphicsStateError(StateQueryError),
}

impl fmt::Display for Sdl2SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Sdl2SurfaceError::InitError(ref e) => write!(f, "initialization error: {}", e),
            Sdl2SurfaceError::WindowCreationFailed(ref e) => {
                write!(f, "failed to create window: {}", e)
            }
            Sdl2SurfaceError::GlContextInitFailed(ref e) => {
                write!(f, "failed to create OpenGL context: {}", e)
            }
            Sdl2SurfaceError::VideoInitError(ref e) => {
                write!(f, "failed to initialize video system: {}", e)
            }
            Sdl2SurfaceError::GraphicsStateError(ref e) => {
                write!(f, "failed to get graphics state: {}", e)
            }
        }
    }
}

/// A [luminance] GraphicsContext backed by SDL2 and OpenGL 3.3 Core.
///
/// ```ignore
/// use luminance_sdl2::Sdl2Surface;
///
/// let surface = Sdl2Surface::build_with(|video| video.window("My app", 800, 600))
///     .expect("failed to create surface");
///
/// let sdl = surface.sdl();
/// ```
///
/// [luminance]: https://crates.io/crates/luminance
pub struct Sdl2Surface {
    sdl: sdl2::Sdl,
    window: sdl2::video::Window,
    gl: GL33,
    // This struct needs to stay alive until we are done with OpenGL stuff.
    _gl_context: sdl2::video::GLContext,
}

impl Sdl2Surface {
    /// Create a new [`Sdl2Surface`] from a [`sdl2::video::WindowBuilder`].
    ///
    /// The callback is passed a reference to [`sdl2::VideoSubsystem`].
    /// This is your chance to change GL attributes before creating the window with your preferred
    /// settings.
    ///
    /// ```ignore
    /// use luminance_sdl2::Sdl2Surface;
    ///
    /// let surface = Sdl2Surface::build_with(|video| {
    ///     let gl_attr = video.gl_attr();
    ///     gl_attr.set_multisample_buffers(1);
    ///     gl_attr.set_multisample_samples(4);
    ///
    ///     let mut builder = video.window("My app", 800, 600);
    ///     builder.fullscreen_desktop();
    ///     builder
    /// })
    ///   .expect("failed to build window");
    /// ```
    pub fn build_with<WB>(window_builder: WB) -> Result<Self, Sdl2SurfaceError>
    where
        WB: FnOnce(&sdl2::VideoSubsystem) -> sdl2::video::WindowBuilder,
    {
        let sdl = sdl2::init().map_err(Sdl2SurfaceError::InitError)?;

        let video_system = sdl.video().map_err(Sdl2SurfaceError::VideoInitError)?;

        let gl_attr = video_system.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_flags().forward_compatible().set();
        gl_attr.set_context_major_version(3);
        gl_attr.set_context_minor_version(3);

        let window = window_builder(&video_system)
            .opengl()
            .build()
            .map_err(Sdl2SurfaceError::WindowCreationFailed)?;

        let _gl_context = window
            .gl_create_context()
            .map_err(Sdl2SurfaceError::GlContextInitFailed)?;

        gl::load_with(|s| video_system.gl_get_proc_address(s) as *const c_void);

        let gl = GL33::new().map_err(Sdl2SurfaceError::GraphicsStateError)?;
        let surface = Sdl2Surface {
            sdl,
            window,
            gl,
            _gl_context,
        };

        Ok(surface)
    }

    /// The entry point to most of the SDL2 API.
    pub fn sdl(&self) -> &sdl2::Sdl {
        &self.sdl
    }

    /// The underlying SDL2 window of this surface.
    pub fn window(&self) -> &sdl2::video::Window {
        &self.window
    }

    /// The underlying SDL2 window of this surface.
    pub fn window_mut(&mut self) -> &mut sdl2::video::Window {
        &mut self.window
    }

    /// Get the back buffer.
    pub fn back_buffer(&mut self) -> Result<Framebuffer<GL33, Dim2, (), ()>, FramebufferError> {
        let (w, h) = self.window.drawable_size();
        Framebuffer::back_buffer(self, [w, h])
    }
}

unsafe impl GraphicsContext for Sdl2Surface {
    type Backend = GL33;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.gl
    }
}
