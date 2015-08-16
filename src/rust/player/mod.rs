use std::default::Default;

use piston::input::*;
use opengl_graphics::Texture as OpenGlTexture;
use collisions::HasBounds;
use collisions;

use super::PlayerGraphics;
use map::Map;

use self::SingleInputState::*;

pub const PLAYER_IMAGE_WIDTH: u32 = 32;
pub const PLAYER_IMAGE_HEIGHT: u32 = 20;
pub const PLAYER_IMAGE_Y_OFFSET: f64 = 0.0;
pub const PLAYER_IMAGE_X_OFFSET: f64 = -11.0;
pub const PLAYER_COLLISION_WIDTH: u32 = 10;
pub const PLAYER_COLLISION_HEIGHT: u32 = 20;

#[derive(Copy, Clone)]
pub enum SingleInputState {
    Pressed,
    PressedAndSinceReleased,
    Released,
}

impl Default for SingleInputState {
    fn default() -> SingleInputState {
        SingleInputState::Released
    }
}

impl SingleInputState {
    fn pressed(&mut self) {
        *self = Pressed;
    }

    fn released(&mut self) {
        if let Pressed = *self {
            *self = PressedAndSinceReleased;
        }
    }

    /// Returns true for PressedAndSinceReleased and Pressed, false for Released
    fn was_pressed(&mut self) -> bool {
        if let Released = *self {
            false
        } else {
            if let PressedAndSinceReleased = *self {
                *self = Released;
            }
            true
        }
    }

    fn consume_press(&mut self) -> bool {
        if let Released = *self {
            false
        } else {
            *self = Released;
            true
        }
    }
}

#[derive(Default)] // everything unpressed
pub struct InputState {
    pub up: SingleInputState,
    pub down: SingleInputState,
    pub left: SingleInputState,
    pub right: SingleInputState,
}

impl InputState {
    fn pressed(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Up) => self.up.pressed(),
            Button::Keyboard(Key::Down) => self.down.pressed(),
            Button::Keyboard(Key::Left) => self.left.pressed(),
            Button::Keyboard(Key::Right) => self.right.pressed(),
            _ => (),
        }
    }

    fn released(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Up) => self.up.released(),
            Button::Keyboard(Key::Down) => self.down.released(),
            Button::Keyboard(Key::Left) => self.left.released(),
            Button::Keyboard(Key::Right) => self.right.released(),
            _ => (),
        }
    }
}

#[derive(Copy, Clone)]
pub enum MovementState {
    MovingLeft,
    MovingRight,
    StillLeft,
    StillRight,
}

impl Default for MovementState {
    fn default() -> MovementState {
        MovementState::StillRight
    }
}

impl MovementState {
    fn set_still(&mut self) {
        match *self {
            MovementState::MovingLeft => *self = MovementState::StillLeft,
            MovementState::MovingRight => *self = MovementState::StillRight,
            _ => (),
        }
    }
}

#[derive(Default)] // everything 0.0
pub struct Player {
    pub absolute_x: f64,
    pub absolute_y: f64,
    pub last_movement: MovementState,
    pub last_grounded: bool,
    pub y_velocity: f64,
    pub input: InputState,
}

impl Player {
    pub fn new(x: f64, y: f64) -> Player {
        Player {
            absolute_x: x,
            absolute_y: y,
            .. Player::default()
        }
    }

    fn update(&mut self, args: &UpdateArgs, map: &Map) {
        let time = args.dt;
        let mut new_x = self.absolute_x;
        let mut new_y = self.absolute_y;
        match (self.input.left.was_pressed(), self.input.right.was_pressed()) {
            (false, true) => {
                new_x += 100.0 * time;
                self.last_movement = MovementState::MovingRight;
            },
            (true, false) => {
                new_x -= 100.0 * time;
                self.last_movement = MovementState::MovingLeft;
            },
            (_, _) => self.last_movement.set_still(),
        }
        new_y -= 200.0 * time;
        new_y += self.y_velocity * time;
        self.y_velocity = (self.y_velocity + 500.0) * 0.2f64.powf(time) - 500.0;

        let collisions = self.collides(new_x, new_y,
            map.blocks().iter().chain(map.boundary_collision_lines().iter()));

        self.last_grounded = collisions.south.is_some();

        match (collisions.west, new_x <= self.absolute_x,
                collisions.east, new_x >= self.absolute_x) {
            (Some(w1), _, Some(w2), _) => {
                self.absolute_x = (w1 + w2) / 2.0;
                self.last_movement.set_still();
            },
            (Some(wall), true, None, _) | (None, _, Some(wall), true) => {
                self.absolute_x = wall;
                self.last_movement.set_still();
            },
            (_, _, _, _) => {
                self.absolute_x = new_x;
            },
        }
        match (collisions.south, new_y <= self.absolute_y,
                collisions.north, new_y >= self.absolute_y) {
            (Some(w1), _, Some(_), _) => {
                self.absolute_y = w1;
                self.y_velocity = 0.0;
            },
            (Some(wall), true, None, _) | (None, _, Some(wall), true) => {
                self.absolute_y = wall;
                self.y_velocity = 0.0;
            },
            (_, _, _, _) => {
                self.absolute_y = new_y;
            },
        }

        if self.input.up.consume_press() && collisions.south.is_some() {
            self.y_velocity += 500.0;
        }
    }

    pub fn event(&mut self, event: &Event, map: &Map) {
        event.press(|b| self.input.pressed(b));
        event.release(|b| self.input.released(b));
        event.update(|args| self.update(args, map));
    }

    pub fn get_current_image<'a>(&self, cache: &'a PlayerGraphics) -> &'a OpenGlTexture {
        if self.last_grounded {
            match self.last_movement {
                MovementState::StillLeft => {
                    &cache.standing_left
                },
                MovementState::StillRight => {
                    &cache.standing_right
                },
                MovementState::MovingLeft => {
                    &cache.run_left[5 - self.absolute_x.ceil() as usize / 4 % 6]
                },
                MovementState::MovingRight => {
                    &cache.run_right[self.absolute_x.ceil() as usize / 4 % 6]
                },
            }
        } else {
            match self.last_movement {
                MovementState::StillLeft | MovementState::MovingLeft => {
                    &cache.run_left[0]
                },
                MovementState::StillRight | MovementState::MovingRight => {
                    &cache.run_right[0]
                }
            }
        }
    }
}

impl collisions::HasBounds for Player {
    fn min_x(&self) -> f64 {
        self.absolute_x
    }

    fn min_y(&self) -> f64 {
        self.absolute_y
    }

    fn len_x(&self) -> f64 {
        PLAYER_COLLISION_WIDTH as f64
    }

    fn len_y(&self) -> f64 {
        PLAYER_COLLISION_HEIGHT as f64
    }
}
