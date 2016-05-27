use std::sync::mpsc;

use piston::input::*;

use collisions::HasBounds;
use collisions;

use map::Map;
use super::Window;

#[derive(Copy, Clone, Debug)]
pub enum SettingsUpdate {
    Weight(f64),
    InputForce(f64),
    JumpBoost(f64),
    WallBoostX(f64),
    WallBoostY(f64),
    GravityForce(f64),
    DragConstant(f64),
    TickConstant(f64),
    JumpDuration(f64),
}

pub struct PlayerSettings<'a> {
    pub weight: f64,
    pub input_force: f64,
    pub jump_boost: f64,
    pub wall_boost_y: f64,
    pub wall_boost_x: f64,
    pub gravity_force: f64,
    pub drag_constant: f64,
    pub tick_constant: f64,
    pub jump_duration: f64,
    pub update_channel: Option<&'a mut mpsc::Receiver<SettingsUpdate>>,
}

impl<'a> Default for PlayerSettings<'a> {
    fn default() -> PlayerSettings<'a> {
        PlayerSettings {
            weight: 3.0,
            input_force: 367.0,
            jump_boost: 400.0,
            wall_boost_y: 250.0,
            wall_boost_x: 450.0,
            gravity_force: 305.0,
            drag_constant: 0.08,
            tick_constant: 3.7,
            jump_duration: 20.0,
            update_channel: None,
        }
    }
}

impl<'a> PlayerSettings<'a> {
    fn new<'b>(sc: &'b mut ::SettingsChannel) -> PlayerSettings<'b> {
        PlayerSettings { update_channel: Some(sc), ..PlayerSettings::default() }
    }

    fn get_updates(&mut self) {
        if let Some(channel) = self.update_channel.as_mut() {
            loop {
                match channel.try_recv() {
                    Ok(update) => {
                        match update {
                            SettingsUpdate::Weight(v) => self.weight = v,
                            SettingsUpdate::InputForce(v) => self.input_force = v,
                            SettingsUpdate::JumpBoost(v) => self.jump_boost = v,
                            SettingsUpdate::WallBoostX(v) => self.wall_boost_x = v,
                            SettingsUpdate::WallBoostY(v) => self.wall_boost_y = v,
                            SettingsUpdate::GravityForce(v) => self.gravity_force = v,
                            SettingsUpdate::DragConstant(v) => self.drag_constant = v,
                            SettingsUpdate::TickConstant(v) => self.tick_constant = v,
                            SettingsUpdate::JumpDuration(v) => self.jump_duration = v,
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        return;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        }
        // Will only break for disconnected
        self.update_channel = None;
    }
}

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


// #[derive(Copy, Clone, PartialEq, Eq, Debug)]
// enum EffectType {
//     Jumping,
// }

// pub struct Effect {
//     pub time_remaining: f64,
//     effect: EffectType,
// }

// impl Effect {
//     pub fn effect(&mut self, player: &mut PlayerState, time_changed: f64) {
//         let delta_time = if self.time_remaining < time_changed {
//             self.time_remaining
//         } else {
//             time_changed
//         };

//         match self.effect {
//             EffectType::Jumping => {
//                 // if player.grounded {
//                 //     self.time_remaining = 0.0;
//                 //     return;
//                 // } else {
//                 //     player.velocity_y += player.settings.jump_force * delta_time;
//                 // }
//             }
//         }
//         self.time_remaining -= delta_time;
//     }
// }

#[derive(Default)]
pub struct PlayerState<'a> {
    pub grounded: bool,
    pub on_left_wall: bool,
    pub on_right_wall: bool,
    pub absolute_x: f64,
    pub absolute_y: f64,
    pub last_movement: MovementState,
    velocity_x: f64,
    velocity_y: f64,
    // current_effects: Vec<Effect>,
    input_left: bool,
    input_right: bool,
    settings: PlayerSettings<'a>,
}

impl<'a> PlayerState<'a> {
    pub fn new<'b>(x: f64, y: f64, sc: &'b mut ::SettingsChannel) -> PlayerState<'b> {
        PlayerState {
            absolute_x: x,
            absolute_y: y,
            settings: PlayerSettings::new(sc),
            ..PlayerState::default()
        }
    }

    fn tick(&mut self, args: &UpdateArgs, map: &Map) {
        let delta_time = args.dt;

        self.settings.get_updates();

        // {
        //     let mut effects = Vec::new();
        //     mem::swap(&mut effects, &mut self.current_effects);
        //     for effect in &mut effects {
        //         effect.effect(self, delta_time);
        //     }
        //     effects.retain(|effect| effect.time_remaining > 0.0);
        //     mem::swap(&mut effects, &mut self.current_effects);
        // }

        let mut force_x = 0.0;
        let mut force_y = 0.0;

        match (self.input_left, self.input_right) {
            (false, true) => {
                force_x += self.settings.input_force;
                self.last_movement = MovementState::MovingRight;
            }
            (true, false) => {
                force_x -= self.settings.input_force;
                self.last_movement = MovementState::MovingLeft;
            }
            (_, _) => self.last_movement.set_still(),
        }

        if !self.grounded {
            force_y -= self.settings.gravity_force;
        }

        force_x -= self.settings.drag_constant * self.velocity_x * self.velocity_x.abs();
        force_y -= self.settings.drag_constant * self.velocity_y * self.velocity_y.abs();

        self.velocity_x += force_x / self.settings.weight * delta_time
                    * self.settings.tick_constant;
        self.velocity_y += force_y / self.settings.weight * delta_time
                    * self.settings.tick_constant;

        let new_x = self.absolute_x + self.velocity_x * delta_time * self.settings.tick_constant;
        let new_y = self.absolute_y + self.velocity_y * delta_time * self.settings.tick_constant;

        let collisions = self.collides(new_x, new_y,
            map.blocks().iter().chain(map.boundary_collision_lines().iter()));

        self.grounded = collisions.south.is_some();
        self.on_left_wall = collisions.west.is_some();
        self.on_right_wall = collisions.east.is_some();

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
                self.velocity_y = 0.0;
            }
            (Some(wall), true, None, _) | (None, _, Some(wall), true) => {
                self.absolute_y = wall;
                self.velocity_y = 0.0;
            }
            (_, _, _, _) => {
                self.absolute_y = new_y;
            }
        }
    }

    fn jump(&mut self) {
        if self.grounded {
            self.velocity_y += self.settings.jump_boost;
        } else if self.on_left_wall {
            self.velocity_x += self.settings.wall_boost_x;
            self.velocity_y += self.settings.wall_boost_y;
            self.last_movement = MovementState::MovingRight;
        } else if self.on_right_wall {
            self.velocity_x -= self.settings.wall_boost_x;
            self.velocity_y += self.settings.wall_boost_y;
            self.last_movement = MovementState::MovingLeft;
        }
    }

    pub fn update(&mut self, event: &Window, map: &Map) {
        event.press(|button| {
            match button {
                Button::Keyboard(Key::Up) => self.jump(),
                Button::Keyboard(Key::Left) => self.input_left = true,
                Button::Keyboard(Key::Right) => self.input_right = true,
                _ => (),
            }
        });
        event.release(|button| {
            match button {
                Button::Keyboard(Key::Left) => self.input_left = false,
                Button::Keyboard(Key::Right) => self.input_right = false,
                _ => (),
            }
        });
        event.update(|args| self.tick(args, map));
    }
}

pub const PLAYER_COLLISION_WIDTH: u32 = 10;
pub const PLAYER_COLLISION_HEIGHT: u32 = 20;

impl<'a> collisions::HasBounds for PlayerState<'a> {
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
