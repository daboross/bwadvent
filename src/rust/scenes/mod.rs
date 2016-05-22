mod play;
mod editor;

use std::ops::Deref;
use std::f64;
use std::fs;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use piston::input::{Button, Key, PressEvent, RenderEvent};
use graphics::{self, Context, Transformed};
use graphics::types::Color;
use graphics::character::CharacterCache;

use super::{Graphics, GraphicsCache, Window, SettingsChannel};

pub type SceneRunFn<'a> = for<'b, 'c, 'd, 'e> Fn(&'b Window, &'c mut Graphics, &'d mut GraphicsCache, &'e mut SettingsChannel) + Sync + 'a;

pub static MAIN_MENU: MenuScene<'static> = MenuScene {
    title: "B/W ADVENTURES",
    options: &[
        ("PLAY", &play_scene as &SceneRunFn),
        ("EDIT", &editor_scene as &SceneRunFn),
    ],
};

fn find_level_dir() -> PathBuf {
    let cwd = current_dir().unwrap();
    let mut current_dir: &Path = &cwd;
    loop {
        for item in fs::read_dir(&current_dir).unwrap() {
            let item = item.unwrap();
            let name = item.file_name();

            let file_type = match item.file_type() {
                Ok(v) => v,
                Err(_) => break,
            };

            if &name[..] == "maps" && file_type.is_dir() {
                let path = item.path();
                // to test permissions
                match fs::symlink_metadata(&path) {
                    Ok(_) => return path,
                    Err(_) => break,
                };
            }
        }
        current_dir = current_dir.parent().expect("Reached filesystem root in search for maps dir");
    }
}

fn play_scene(window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
        sc: &mut SettingsChannel) {
    let level_dir = find_level_dir();

    let play_options = fs::read_dir(&level_dir).unwrap().filter_map(Result::ok).map(|i| i.path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("map")).map(|path| {
        // unwrap here because DirEntry guarantees that there will be a file name.
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();

        (name, Box::new(move |window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
                sc: &mut SettingsChannel| {
            play::PlayScene::new(&path).run(window, graphics, cache, sc);
        }) as Box<Fn(&Window, &mut Graphics, &mut GraphicsCache, &mut SettingsChannel) + Sync>)
    }).collect::<Vec<_>>();

    let menu = MenuScene { title: "CHOOSE LEVEL", options: &play_options[..] };

    menu.run(window, graphics, cache, sc);
}

fn editor_scene(window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
        sc: &mut SettingsChannel) {
    let level_dir = find_level_dir();

    let editor_options = fs::read_dir(&level_dir).unwrap().filter_map(Result::ok).map(|i| i.path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("map")).map(|path| {
        // unwrap here because DirEntry guarantees that there will be a file name.
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();

        (name, Box::new(move |window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
                sc: &mut SettingsChannel| {
            editor::EditorScene::new(&path).run(window, graphics, cache, sc);
        }) as Box<Fn(&Window, &mut Graphics, &mut GraphicsCache, &mut SettingsChannel) + Sync>)
    }).collect::<Vec<_>>();

    let menu = MenuScene { title: "CHOOSE LEVEL", options: &editor_options[..] };

    menu.run(window, graphics, cache, sc);
}


fn draw_text<T: AsRef<str>>(position: [f64; 4], text: T, text_size: u32, color: Color,
                            cache: &mut GraphicsCache, context: &Context, graphics: &mut Graphics) {
    let x_pos = position[0];
    let y_pos = position[1];
    let width = position[2];
    let height = position[3];
    let text = text.as_ref();
    if let Some(first_char) = text.chars().next() {
        let (text_height, text_offset_top) = {
            let graphics_char = cache.font.character(text_size, first_char);
            (graphics_char.height(), graphics_char.top())
        };
        let mut text_width = 0.0;
        for c in text.chars() {
            text_width += cache.font.character(text_size, c).width();
        }

        graphics::Text::new_color(color, text_size).draw(
            text,
            &mut cache.font,
            &context.draw_state,
            context.trans(
                (x_pos + width / 2.0 - text_width / 2.0).floor(),
                (y_pos + height / 2.0 - text_height / 2.0 + text_offset_top / 2.0).floor()
            ).transform,
            graphics
        );
    }
}

pub struct MenuScene<'a, TiT = &'a str, OpT = &'a str, FnT = &'a SceneRunFn<'a>>
        where TiT: AsRef<str> + 'a,
                OpT: AsRef<str> + 'a,
                FnT: Deref<Target=SceneRunFn<'a>> + 'a {
    title: TiT,
    options: &'a [(OpT, FnT)],
}

impl<'a, TiT, OpT, FnT> MenuScene<'a, TiT, OpT, FnT>
        where TiT: AsRef<str> + 'a,
                OpT: AsRef<str> + 'a,
                FnT: Deref<Target=SceneRunFn<'a>> + 'a {
    pub fn run(&self, window: &Window, graphics: &mut Graphics, cache: &mut GraphicsCache,
            sc: &mut SettingsChannel) {
        let mut selected = 0usize;

        for event in window.clone() {
            if let Some(Button::Keyboard(Key::Escape)) = event.press_args() {
                break;
            }

            event.render(|event| {
                let screen_width = event.width as f64;
                let screen_height = event.height as f64;

                let viewport = graphics::Viewport {
                    rect: [0, 0, event.width as i32, event.height as i32],
                    draw_size: [1; 2],
                    window_size: [1; 2],
                };

                graphics.draw(viewport, |context, graphics| {
                    graphics::clear(graphics::color::BLACK, graphics);
                    let width = f64::min(screen_width * 0.8,  400.0).floor();
                    let height = f64::min(screen_height * 0.8 / ((self.options.len() + 2) as f64 *
                            1.2), 20.0).floor();
                    let x_pos = ((screen_width - width) / 2.0).floor();

                    draw_text(
                        [x_pos, screen_height * 0.2, width, height * 1.5],
                        &self.title,
                        (height * 1.5) as u32,
                        graphics::color::WHITE,
                        cache, &context, graphics,
                    );

                    for (index, &(ref text, _)) in self.options.iter().enumerate() {
                        let y_pos = (screen_height * 0.2 + (index + 2) as f64 * height * 1.2
                                ).floor();
                        let color = if index == selected {
                            graphics::color::WHITE
                        } else {
                            graphics::color::grey(0.2)
                        };
                        graphics::Rectangle::new(color).draw(
                            [x_pos, y_pos, width, height],
                            &context.draw_state,
                            context.transform,
                            graphics,
                        );

                        draw_text(
                            [x_pos, y_pos, width, height],
                            text,
                            (height * 0.8) as u32,
                            graphics::color::BLACK,
                            cache, &context, graphics,
                        );
                    }
                })
            });

            event.press(|b| {
                match b {
                    Button::Keyboard(Key::Up) => {
                        if selected == 0 { // selected is usize
                            selected = self.options.len() - 1;
                        } else {
                            selected -= 1;
                        }
                    }
                    Button::Keyboard(Key::Down) => {
                        selected += 1;
                        selected %= self.options.len();
                    }
                    Button::Keyboard(Key::Return) => {
                        println!("Selected: {}", selected);
                        (self.options[selected].1)(window, graphics, cache, sc);
                    }
                    _ => (),
                }
            });
        }
    }
}
