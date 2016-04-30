use std::default::Default;

use piston::input::*;
use opengl_graphics::Texture as OpenGlTexture;
use collisions::HasBounds;
use collisions;

use super::Window;
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

    fn is_pressed(&self) -> bool {
        if let Pressed = *self {
            true
        } else {
            false
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

#[derive(Copy, Clone, PartialEq, Eq)]
enum EffectType {
    Jumping,
}

pub struct Effect {
    pub time_remaining: f64,
    effect: EffectType,
}

impl Effect {
    pub fn effect(&mut self, player: &mut Player, time_changed: f64) {
        let change = if self.time_remaining < time_changed {
            self.time_remaining
        } else {
            time_changed
        };

        match self.effect {
            EffectType::Jumping => {
                if player.grounded {
                    self.time_remaining = 0.0;
                    return;
                } else {
                    player.y_velocity += 1000.0 * change;
                }
            }
        }
        self.time_remaining -= change;
    }
}

#[derive(Default)] // everything 0.0
pub struct Player {
    pub grounded: bool,
    pub last_scroll_x: f64,
    pub last_scroll_y: f64,
    pub absolute_x: f64,
    pub absolute_y: f64,
    pub last_movement: MovementState,
    pub y_velocity: f64,
    pub x_velocity: f64,
    pub input: InputState,
    pub current_effects: Option<Vec<Effect>>,
}

impl Player {
    pub fn new(x: f64, y: f64) -> Player {
        Player { absolute_x: x, absolute_y: y, ..Player::default() }
    }

    fn update(&mut self, args: &UpdateArgs, map: &Map) {
        let time = args.dt;

        // maybe some better way to avoid multiple borrows than an Option?
        if let Some(mut effects) = self.current_effects.take() {
            for effect in &mut effects {
                effect.effect(self, time);
            }
            effects.retain(|effect| effect.time_remaining > 0.0);
            self.current_effects = Some(effects);
        }
        match (self.input.left.was_pressed(), self.input.right.was_pressed()) {
            (false, true) => {
                if self.grounded {
                    self.x_velocity += 1000.0 * time;
                } else {
                    self.x_velocity += 500.0 * time;
                }
                self.last_movement = MovementState::MovingRight;
            }
            (true, false) => {
                if self.grounded {
                    self.x_velocity -= 1000.0 * time;
                } else {
                    self.x_velocity -= 500.0 * time;
                }
                self.last_movement = MovementState::MovingLeft;
            }
            (_, _) => self.last_movement.set_still(),
        }

        if self.grounded {
            if self.input.left.is_pressed() || self.input.right.is_pressed() {
                self.x_velocity *= 0.7f64.powf(time * 20.0);
            } else {
                self.x_velocity *= 0.2f64.powf(time * 20.0);
            }
        } else {
            self.x_velocity *= 0.8f64.powf(time * 20.0);
        }

        if !self.grounded {
            self.y_velocity -= 2000.0 * time;
        }

        self.y_velocity = self.y_velocity * 0.7f64.powf(time * 20.0);

        let new_y = self.absolute_y + self.y_velocity * time;
        let new_x = self.absolute_x + self.x_velocity * time;

        let collisions = self.collides(new_x, new_y,
            map.blocks().iter().chain(map.boundary_collision_lines().iter()));

        self.grounded = collisions.south.is_some();

        match (collisions.west, new_x <= self.absolute_x,
                collisions.east, new_x >= self.absolute_x) {
            (Some(w1), _, Some(w2), _) => {
                self.absolute_x = (w1 + w2) / 2.0;
                self.last_movement.set_still();
            }
            (Some(wall), true, None, _) | (None, _, Some(wall), true) => {
                self.absolute_x = wall;
                self.last_movement.set_still();
            }
            (_, _, _, _) => {
                self.absolute_x = new_x;
            }
        }
        match (collisions.south, new_y <= self.absolute_y,
                collisions.north, new_y >= self.absolute_y) {
            (Some(south_wall), _, Some(_), _) => {
                self.absolute_y = south_wall;
                self.y_velocity = 0.0;
            }
            (Some(wall), true, None, _) | (None, _, Some(wall), true) => {
                self.absolute_y = wall;
                self.y_velocity = 0.0;
            }
            (_, _, _, _) => {
                self.absolute_y = new_y;
            }
        }

        fn jump(player: &mut Player) {
            player.y_velocity += 1000.0;
            if let None = player.current_effects {
                player.current_effects = Some(Vec::new());
            }

            player.current_effects.as_mut().unwrap().retain(|e| e.effect != EffectType::Jumping);
            player.current_effects.as_mut().unwrap().push(Effect {
                time_remaining: 10.0,
                effect: EffectType::Jumping,
            });
        }

        if self.input.up.consume_press() {
            if collisions.south.is_some() {
                jump(self);
            } else if collisions.west.is_some() {
                jump(self);
                self.x_velocity += 500.0;
                self.last_movement = MovementState::MovingRight;
            } else if collisions.east.is_some() {
                jump(self);
                self.x_velocity -= 500.0;
                self.last_movement = MovementState::MovingLeft;
            }
        }
    }

    pub fn event(&mut self, event: &Window, map: &Map) {
        event.press(|b| self.input.pressed(b));
        event.release(|b| self.input.released(b));
        event.update(|args| self.update(args, map));
    }

    pub fn get_current_image<'a>(&self, cache: &'a PlayerGraphics) -> &'a OpenGlTexture {
        if self.grounded {
            match self.last_movement {
                MovementState::StillLeft => {
                    &cache.standing_left
                }
                MovementState::StillRight => {
                    &cache.standing_right
                }
                MovementState::MovingLeft => {
                    &cache.run_left[5 - self.absolute_x.ceil() as usize / 4 % 6]
                }
                MovementState::MovingRight => {
                    &cache.run_right[self.absolute_x.ceil() as usize / 4 % 6]
                }
            }
        } else {
            match self.last_movement {
                MovementState::StillLeft | MovementState::MovingLeft => {
                    &cache.run_left[0]
                }
                MovementState::StillRight | MovementState::MovingRight => {
                    &cache.run_right[0]
                }
            }
        }
    }

    /// Takes screen width and height, gives (scroll_x, scroll_y)
    pub fn calculate_scroll(&mut self, width: f64, height: f64) -> (f64, f64) {
        const ALLOWANCE: f64 = 100.0;

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let scroll_x = if self.absolute_x > self.last_scroll_x + half_width - ALLOWANCE {
            self.absolute_x - half_width + ALLOWANCE
        } else if self.absolute_x < self.last_scroll_x + ALLOWANCE - width / 2.0 {
            self.absolute_x - ALLOWANCE + width / 2.0
        } else {
            self.last_scroll_x
        };

        let scroll_y = if self.absolute_y > self.last_scroll_y + half_height - ALLOWANCE {
            self.absolute_y - half_height + ALLOWANCE
        } else if self.absolute_y < self.last_scroll_y + ALLOWANCE - half_height {
            self.absolute_y - ALLOWANCE + half_height
        } else {
            self.last_scroll_y
        };

        self.last_scroll_x = scroll_x;
        self.last_scroll_y = scroll_y;

        (scroll_x.floor(), scroll_y.floor())
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
