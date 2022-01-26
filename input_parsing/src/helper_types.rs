use bevy::utils::HashSet;
use types::{GameButton, StickPosition};

#[derive(Clone, PartialEq, Debug, Default)]
/// Frame is a situation, diff is a change
pub struct Frame {
    pub stick_position: StickPosition,
    pub pressed: HashSet<GameButton>,
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

    pub fn diff_from_neutral(&self) -> Diff {
        let stick_move = if self.stick_position == StickPosition::Neutral {
            None
        } else {
            Some(self.stick_position)
        };

        let pressed = if self.pressed.is_empty() {
            None
        } else {
            Some(self.pressed.clone())
        };

        Diff {
            stick_move,
            pressed,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
/// A single update in input state
pub struct Diff {
    pub stick_move: Option<StickPosition>,
    pub pressed: Option<HashSet<GameButton>>,
    pub released: Option<HashSet<GameButton>>,
}
impl Diff {
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

    pub fn released_contains(&self, button: &GameButton) -> bool {
        if let Some(released) = &self.released {
            released.contains(button)
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
