use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

use piston::input::{Button, Event, Key, PressEvent, RenderArgs, RenderEvent};
use piston::event_loop::Events;
use graphics::{ImageSize, Transformed};

use super::super::{Graphics, GraphicsCache, Window, graphics};
use level_serialization::{Level, load_level};
use map::Map;
use player::{PLAYER_IMAGE_X_OFFSET, PLAYER_IMAGE_Y_OFFSET, Player};

pub struct PlayScene {
    map: Level,
}

impl PlayScene {
    pub fn new<T: AsRef<Path> + Copy>(level_file: T) -> PlayScene {
        let mut buf = Vec::new();
        {
            let mut file = File::open(level_file).unwrap();
            file.read_to_end(&mut buf).unwrap();
        }
        PlayScene {
            map: load_level(&buf).expect(&format!("Failed to load level: {}",
                level_file.as_ref().display())),
        }
    }

    pub fn run(&self, window: &Rc<RefCell<Window>>, graphics: &mut Graphics, cache: &mut GraphicsCache) {
        let mut session = PlayData::new(&self.map, graphics, cache);

        for event in window.events() {
            if let Some(Button::Keyboard(Key::Escape)) = event.press_args() {
                break;
            }
            session.process(event);
        }
    }
}

struct PlayData<'a> {
    graphics: &'a mut Graphics,
    cache: &'a mut GraphicsCache,
    map: Map,
    player: Player,
}

impl<'a> PlayData<'a> {
    pub fn new<'b>(level: &Level, graphics: &'b mut Graphics, cache: &'b mut GraphicsCache)
                   -> PlayData<'b> {
        let map = Map::from(level.clone());
        PlayData {
            graphics: graphics,
            cache: cache,
            player: Player::new(map.initial_x(), map.initial_y()),
            map: map,
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
