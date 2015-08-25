extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate image;
extern crate time;
#[macro_use] extern crate nom;
extern crate collisions;

mod level_serialization;
mod map;
mod player;

use std::path::Path;
use std::fs;

use piston::input::{RenderArgs, Event, RenderEvent};
use piston::event_loop::Events;
use graphics::{Transformed, ImageSize};

use opengl_graphics::Texture as OpenGlTexture;
use piston::window::WindowSettings;

use player::{
    Player,
    PLAYER_IMAGE_WIDTH,
    PLAYER_IMAGE_HEIGHT,
    PLAYER_IMAGE_X_OFFSET,
    PLAYER_IMAGE_Y_OFFSET,
};
use map::Map;

pub fn run() {
    let opengl_version = opengl_graphics::OpenGL::V3_2;

    let window = glutin_window::GlutinWindow::new(
        WindowSettings::new("b-w-adventures", [640u32, 480u32])
                        .exit_on_esc(true)
                        .opengl(opengl_version)
    ).unwrap();

    let mut app = Application::new(opengl_graphics::GlGraphics::new(opengl_version));

    for e in window.events() {
        app.process(e);
    }
}

pub struct Application {
    graphics: opengl_graphics::GlGraphics,
    cache: GraphicsCache,
    map: Map,
    player: Player,
}

impl Application {
    pub fn new(graphics: opengl_graphics::GlGraphics) -> Application {
        let level = level_serialization::load_level(&include_bytes!("../maps/map.map")[..]);
        let map = map::Map::from(level.unwrap());
        Application {
            graphics: graphics,
            cache: GraphicsCache::load(),
            player: Player::new(map.initial_x(), map.initial_y()),
            map: map
        }
    }

    fn render(&mut self, event: &RenderArgs) {
        let screen_width = event.width as f64;
        let screen_height = event.height as f64;

        // TODO: see if the [1; 2] instead of [0; 2] wants to be included in any example projects
        let viewport = graphics::Viewport {
            rect: [0, 0, event.width as i32, event.height as i32],
            draw_size: [1; 2],
            window_size: [1; 2],
        };

        let (scroll_x, scroll_y) = self.player.calculate_scroll(screen_width, screen_height);
        let cache = &self.cache;
        let player = &self.player;
        let map = &self.map;

        self.graphics.draw(viewport, |context, graphics| {
            let context = context.trans(-scroll_x, scroll_y);
            graphics::clear(graphics::color::BLACK, graphics);
            graphics::Rectangle::new(graphics::color::WHITE).draw(
                map.boundaries(),
                &context.draw_state,
                context.trans(screen_width / 2.0, screen_height / 2.0).flip_v().transform,
                graphics,
            );
            graphics::image(
                player.get_current_image(&cache.player),
                context.trans(
                    screen_width / 2.0 + player.absolute_x.ceil() + PLAYER_IMAGE_X_OFFSET as f64,
                    screen_height / 2.0 - cache.player.get_height() as f64
                        - player.absolute_y.ceil() - PLAYER_IMAGE_Y_OFFSET as f64
                ).transform,
                graphics,
            );
            for block in map.blocks() {
                graphics::Rectangle::new(graphics::color::BLACK).draw(
                    block,
                    &context.draw_state,
                    context.trans(screen_width / 2.0, screen_height / 2.0).flip_v().transform,
                    graphics,
                );
            }
        })
    }

    pub fn process(&mut self, event: Event) {
        event.render(|event| self.render(event));
        self.player.event(&event, &self.map);
    }
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
        let run_left = load_texture_frames("src/png/unarmed/runleft.png", 6);
        let run_right = load_texture_frames("src/png/unarmed/runright.png", 6);
        let standing_left = load_texture("src/png/unarmed/readyleft.png");
        let standing_right = load_texture("src/png/unarmed/readyright.png");
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
}

impl GraphicsCache {
    pub fn load() -> GraphicsCache {
        GraphicsCache {
            player: PlayerGraphics::load(),
        }
    }
}

fn load_texture<T: AsRef<Path> + ?Sized>(path: &T) -> OpenGlTexture {
    let file = fs::File::open(path.as_ref()).unwrap();
    let dynamic_image = image::load(file, image::ImageFormat::PNG).unwrap();
    let rgba_image = dynamic_image.to_rgba();

    opengl_graphics::Texture::from_image(&rgba_image)
}

fn load_texture_frames<T: AsRef<Path> + ?Sized>(path: &T, num_frames: u32) -> Vec<OpenGlTexture> {
    let file = fs::File::open(path).unwrap();
    let mut image = image::load(file, image::ImageFormat::PNG).unwrap().to_rgba();
    let (image_width, height) = image.dimensions();

    assert_eq!(image_width % num_frames, 0);

    let width = image_width / num_frames;

    (0..num_frames).map(|x| {
        let sub_image = image::SubImage::new(&mut image, x * width, 0, width, height);

        opengl_graphics::Texture::from_image(&sub_image.to_image())
    }).collect()
}
