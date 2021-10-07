mod stick_position;
pub use stick_position::StickPosition;
mod motion_input;
pub use motion_input::MotionInput;

use bevy::{prelude::*, utils::HashSet};
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// Buttons of the game
/// The name 'Button' is in prelude
pub enum GameButton {
    Heavy,
    Fast,
}

/// I.E. Quarter circle forward press punch -> fireball
pub struct Special {
    pub motion: MotionInput,
    pub button: Option<GameButton>,
}
impl Special {
    pub fn advance(&mut self, diff: &Diff) -> bool {
        if let Some(stick) = diff.stick_move {
            self.motion.advance(stick);
        }

        if self.motion.is_done() {
            if let Some(button) = &self.button {
                diff.pressed_contains(button)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.motion.clear();
    }
}

pub struct Normal {
    pub button: GameButton,
    pub stick: Option<StickPosition>,
}

#[derive(Clone, PartialEq)]
/// Frame is a situation, diff is a change
pub struct Frame {
    pub timestamp: Instant,
    pub stick_position: StickPosition,
    pub pressed: HashSet<GameButton>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            stick_position: Default::default(),
            pressed: Default::default(),
        }
    }
}
impl Frame {
    pub fn apply(&mut self, diff: Diff) {
        if let Some(stick) = diff.stick_move {
            self.stick_position = stick;
        }

        if let Some(pressed) = diff.pressed {
            self.pressed = self.pressed.union(&pressed).into_iter().cloned().collect();
        }

        if let Some(released) = diff.released {
            self.pressed.retain(|button| !released.contains(button));
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
/// A single update in input state
pub struct Diff {
    pub stick_move: Option<StickPosition>,
    pub pressed: Option<HashSet<GameButton>>,
    pub released: Option<HashSet<GameButton>>,
}
impl Diff {
    pub fn flip(mut self) -> Self {
        if let Some(stick) = self.stick_move {
            self.stick_move = Some(stick.flip());
        }
        self
    }

    pub fn apply(mut self, change: &InputChange) -> Self {
        match change {
            InputChange::Button(button, update) => match update {
                ButtonUpdate::Pressed => self.pressed = Some(add_or_init(self.pressed, *button)),
                ButtonUpdate::Released => self.released = Some(add_or_init(self.released, *button)),
            },
            InputChange::Stick(stick) => {
                self.stick_move = Some(*stick);
            }
        }

        self
    }

    pub fn pressed_contains(&self, button: &GameButton) -> bool {
        if let Some(pressed) = &self.pressed {
            pressed.contains(button)
        } else {
            false
        }
    }
}
fn add_or_init(base: Option<HashSet<GameButton>>, button: GameButton) -> HashSet<GameButton> {
    if let Some(mut pressed) = base {
        pressed.insert(button);
        pressed
    } else {
        vec![button].into_iter().collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonUpdate {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub enum InputChange {
    Button(GameButton, ButtonUpdate),
    Stick(StickPosition),
}

#[derive(Clone)]
pub struct OwnedChange {
    pub controller: Gamepad,
    pub change: InputChange,
}