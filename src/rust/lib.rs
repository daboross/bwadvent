extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston_window;
extern crate image;
extern crate time;
#[macro_use]
extern crate nom;
extern crate collisions;

mod level_serialization;
mod map;
mod player;
mod scenes;
mod mechanics;

use graphics::ImageSize;
use opengl_graphics::glyph_cache::GlyphCache;

use opengl_graphics::Texture as OpenGlTexture;
use opengl_graphics::TextureSettings;
use piston::window::WindowSettings;

use player::{PLAYER_IMAGE_HEIGHT, PLAYER_IMAGE_WIDTH};

pub type Window = piston_window::PistonWindow;
pub type Graphics = opengl_graphics::GlGraphics;

pub fn run() {
    let opengl_version = opengl_graphics::OpenGL::V3_0;

    let window = WindowSettings::new("b-w-adventures", [640u32, 480u32])
                        .exit_on_esc(false)
                        .srgb(false)
                        .opengl(opengl_version)
                        .build().unwrap();

    let mut graphics = opengl_graphics::GlGraphics::new(opengl_version);
    let mut cache = GraphicsCache::load();

    scenes::MAIN_MENU.run(&window, &mut graphics, &mut cache)
}

pub struct PlayerGraphics {
    run_left: Vec<OpenGlTexture>,
    run_right: Vec<OpenGlTexture>,
    standing_left: OpenGlTexture,
    standing_right: OpenGlTexture,
    /// width, height
    dimensions: (u32, u32),
}

impl PlayerGraphics {
    pub fn load() -> PlayerGraphics {
        let run_left = load_texture_frames(&include_bytes!("../png/unarmed/runleft.png")[..], 6);
        let run_right = load_texture_frames(&include_bytes!("../png/unarmed/runright.png")[..], 6);
        let standing_left = load_texture(&include_bytes!("../png/unarmed/readyleft.png")[..]);
        let standing_right = load_texture(&include_bytes!("../png/unarmed/readyright.png")[..]);
        assert_eq!(PLAYER_IMAGE_WIDTH, run_left[0].get_width());
        assert_eq!(PLAYER_IMAGE_WIDTH, run_right[0].get_width());
        assert_eq!(PLAYER_IMAGE_WIDTH, standing_left.get_width());
        assert_eq!(PLAYER_IMAGE_WIDTH, standing_right.get_width());
        assert_eq!(PLAYER_IMAGE_HEIGHT, run_left[0].get_height());
        assert_eq!(PLAYER_IMAGE_HEIGHT, run_right[0].get_height());
        assert_eq!(PLAYER_IMAGE_HEIGHT, standing_left.get_height());
        assert_eq!(PLAYER_IMAGE_HEIGHT, standing_right.get_height());
        PlayerGraphics {
            run_left: run_left,
            run_right: run_right,
            standing_left: standing_left,
            standing_right: standing_right,
            dimensions: (PLAYER_IMAGE_WIDTH, PLAYER_IMAGE_HEIGHT),
        }
    }

    pub fn get_height(&self) -> u32 {
        self.dimensions.1
    }

    pub fn get_width(&self) -> u32 {
        self.dimensions.0
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}


pub struct GraphicsCache {
    player: PlayerGraphics,
    font: GlyphCache<'static>,
}

impl GraphicsCache {
    pub fn load() -> GraphicsCache {
        GraphicsCache {
            player: PlayerGraphics::load(),
            font: GlyphCache::from_bytes(include_bytes!("../ttf/SigmarOne.ttf"))
                      .unwrap(),
        }
    }
}

// fn load_texture<T: AsRef<Path> + ?Sized>(path: &T) -> OpenGlTexture {
//     let file = fs::File::open(path.as_ref()).unwrap();
//     let image = image::load(file, image::ImageFormat::PNG).unwrap().to_rgba();
fn load_texture(bytes: &[u8]) -> OpenGlTexture {
    let image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::PNG)
                    .unwrap()
                    .to_rgba();

    opengl_graphics::Texture::from_image(&image, &TextureSettings::new())
}

// fn load_texture_frames<T: AsRef<Path> + ?Sized>(path: &T, num_frames: u32) -> Vec<OpenGlTexture> {
//     let file = fs::File::open(path).unwrap();
//     let mut image = image::load(file, image::ImageFormat::PNG).unwrap().to_rgba();
fn load_texture_frames(bytes: &[u8], num_frames: u32) -> Vec<OpenGlTexture> {
    let mut image = image::load_from_memory_with_format(bytes.as_ref(), image::ImageFormat::PNG)
                        .unwrap()
                        .to_rgba();
    let (image_width, height) = image.dimensions();

    assert_eq!(image_width % num_frames, 0);

    let width = image_width / num_frames;

    (0..num_frames).map(|x| {
        let sub_image = image::SubImage::new(&mut image, x * width, 0, width, height);

        opengl_graphics::Texture::from_image(&sub_image.to_image(), &TextureSettings::new())
    }).collect()
}
