use std::path::Path;
use std::fs::File;
use std::io::Read;

use piston::input::{Button, Key, MouseButton, PressEvent, RenderEvent, ReleaseEvent,
    MouseCursorEvent};
use graphics::{Transformed, self};

use super::super::{Graphics, GraphicsCache, Window, SettingsChannel};
use super::play::PlayData;
use level_serialization::{Level, load_level};
use map::Platform;

pub struct EditorScene {
    map: Level,
}

impl EditorScene {
    pub fn new<T: AsRef<Path> + Copy>(level_file: T) -> EditorScene {
        let mut buf = Vec::new();
        {
            let mut file = File::open(level_file).unwrap();
            file.read_to_end(&mut buf).unwrap();
        }
        EditorScene {
            map: load_level(&buf).expect(&format!("Failed to load level: {}",
                level_file.as_ref().display())),
        }
    }

    pub fn run(&self, window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
            sc: &mut SettingsChannel) {
        let mut session = EditorData::new(&self.map, graphics, cache, sc);

        for event in window.clone() {
            if let Some(Button::Keyboard(Key::Escape)) = event.press_args() {
                break;
            }
            session.process(&event);
        }
    }
}

struct EditorData<'a> {
    play_data: PlayData<'a>,
    current_mouse_x: f64,
    current_mouse_y: f64,
    screen_width: f64,
    screen_height: f64,
    scroll_start: Option<(f64, f64)>,
}

impl<'a> EditorData<'a> {
    pub fn new<'b>(level: &Level, graphics: &'b mut Graphics, cache: &'b mut GraphicsCache,
                    sc: &'b mut SettingsChannel)
                   -> EditorData<'b> {
        EditorData {
            play_data: PlayData::new(level, graphics, cache, sc),
            current_mouse_x: 0f64,
            current_mouse_y: 0f64,
            screen_width: 0f64,
            screen_height: 0f64,
            scroll_start: None,
        }
    }

    pub fn process(&mut self, event: &Window) {
        self.play_data.process(event);
        event.render(|args| {
            self.screen_width = args.width as f64;
            self.screen_height = args.height as f64;
            if let Some((start_x, start_y)) = self.scroll_start {
                let viewport = graphics::Viewport {
                    rect: [0, 0, args.width as i32, args.height as i32],
                    draw_size: [1; 2],
                    window_size: [1; 2],
                };

                let (scroll_x, scroll_y) = (self.play_data.player.last_scroll_x, self.play_data.player.last_scroll_y);

                let screen_width = args.width as f64;
                let screen_height = args.height as f64;

                let current_x = self.current_mouse_x + scroll_x - screen_width / 2.0;
                let current_y = self.current_mouse_y + scroll_y - screen_height / 2.0;
                let length_x = f64::abs(current_x - start_x);
                let length_y = f64::abs(current_y - start_y);
                let min_x = f64::min(start_x, current_x);
                let min_y = f64::min(start_y, current_y);

                self.play_data.graphics.draw(viewport, |context, graphics| {
                    graphics::Rectangle::new(graphics::color::BLACK).draw(
                        [min_x - scroll_x, min_y - scroll_y, length_x, length_y],
                        &context.draw_state,
                        context.trans(screen_width / 2.0, screen_height / 2.0).flip_v().transform,
                        graphics,
                    );
                })
            }
        });
        event.mouse_cursor(|x, y| {
            self.current_mouse_x = x;
            self.current_mouse_y = self.screen_height - y;
        });
        event.press(|button| {
            if let Button::Mouse(MouseButton::Left) = button {
                let player = &self.play_data.player;
                let scroll_x = player.last_scroll_x;
                let scroll_y = player.last_scroll_y;
                self.scroll_start = Some((
                    self.current_mouse_x + scroll_x - self.screen_width / 2.0,
                    self.current_mouse_y + scroll_y - self.screen_height / 2.0));
            }
        });
        event.release(|button| {
            if let Button::Mouse(MouseButton::Left) = button {
                if let Some((start_x, start_y)) = self.scroll_start.take() {
                    let player = &self.play_data.player;
                    let scroll_x = player.last_scroll_x;
                    let scroll_y = player.last_scroll_y;
                    let current_x = self.current_mouse_x + scroll_x - self.screen_width / 2.0;
                    let current_y = self.current_mouse_y + scroll_y - self.screen_height / 2.0;
                    let min_x = f64::min(start_x, current_x);
                    let max_x = f64::max(start_x, current_x);
                    let min_y = f64::min(start_y, current_y);
                    let max_y = f64::max(start_y, current_y);
                    let len_x = max_x - min_x;
                    let len_y = max_y - min_y;

                    self.play_data.map.add_block(Platform::new_box(
                        min_x, min_y, len_x, len_y
                    ));

                    println!("platform.box: {:.1},{:.1},{:.1},{:.1}", min_x, min_y, len_x, len_y);
                }
            }
        });
    }
}
