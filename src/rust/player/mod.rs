use std::f64;

use piston::input::*;
use opengl_graphics::Texture as OpenGlTexture;

use super::PlayerGraphics;
use super::SettingsChannel;
use map::Map;
use mechanics::PlayerState;
use mechanics::MovementState;

pub const PLAYER_IMAGE_WIDTH: u32 = 32;
pub const PLAYER_IMAGE_HEIGHT: u32 = 20;
pub const PLAYER_IMAGE_Y_OFFSET: f64 = 0.0;
pub const PLAYER_IMAGE_X_OFFSET: f64 = -11.0;

#[derive(Default)]
pub struct Player<'a> {
    pub last_scroll_x: f64,
    pub last_scroll_y: f64,
    pub state: PlayerState<'a>,
}

impl<'a> Player<'a> {
    pub fn new<'b>(x: f64, y: f64, sc: &'b mut SettingsChannel) -> Player<'b> {
        Player {
            state: PlayerState::new(x, y, sc),
            last_scroll_x: x,
            last_scroll_y: y,
        }
    }

    pub fn event(&mut self, event: &Event, map: &Map) {
        self.state.update(event, map);
    }

    pub fn get_current_image<'b>(&self, cache: &'b PlayerGraphics) -> &'b OpenGlTexture {
        if self.state.grounded {
            match self.state.last_movement {
                MovementState::StillLeft => {
                    &cache.standing_left
                }
                MovementState::StillRight => {
                    &cache.standing_right
                }
                MovementState::MovingLeft => {
                    &cache.run_left[5 - self.state.absolute_x.ceil() as usize / 4 % 6]
                }
                MovementState::MovingRight => {
                    &cache.run_right[self.state.absolute_x.ceil() as usize / 4 % 6]
                }
            }
        } else {
            match self.state.last_movement {
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
        let allowance_x = f64::max(width / 5.0, 100.0);
        let allowance_y = f64::max(height / 5.0, 100.0);

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let scroll_x = if self.state.absolute_x > self.last_scroll_x + half_width - allowance_x {
            self.state.absolute_x - half_width + allowance_x
        } else if self.state.absolute_x < self.last_scroll_x + allowance_x - width / 2.0 {
            self.state.absolute_x - allowance_x + width / 2.0
        } else {
            self.last_scroll_x
        };

        let scroll_y = if self.state.absolute_y > self.last_scroll_y + half_height - allowance_y {
            self.state.absolute_y - half_height + allowance_y
        } else if self.state.absolute_y < self.last_scroll_y + allowance_y - half_height {
            self.state.absolute_y - allowance_y + half_height
        } else {
            self.last_scroll_y
        };

        self.last_scroll_x = scroll_x;
        self.last_scroll_y = scroll_y;

        (scroll_x.floor(), scroll_y.floor())
    }
}
