mod direction;
pub use direction::*;

mod inputs;
pub use inputs::{GameButton, StickPosition};

use bevy_inspector_egui::Inspectable;
use std::fmt::{Debug, Display};
use strum_macros::EnumIter;

use bevy::prelude::*;

// This crate will be as small as possible so that types are where they are used
// It's meant for common universal types to circumvent circular dependencies.

pub struct Players {
    pub one: Entity,
    pub two: Entity,
}
impl Players {
    pub fn get(&self, player: Player) -> Entity {
        match player {
            Player::One => self.one,
            Player::Two => self.two,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Owner(pub Player);

#[derive(EnumIter, Inspectable, PartialEq, Eq, Clone, Copy, Debug, Hash, Component)]
pub enum Player {
    One,
    Two,
}
impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Player {
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
}
