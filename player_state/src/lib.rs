use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use types::MoveId;

mod primary_state;

mod player_state;
pub use crate::player_state::PlayerState;

pub const PLAYER_SPRITE_WIDTH: f32 = 0.80;
pub const PLAYER_SPRITE_STANDING_HEIGHT: f32 = 1.80;
const PLAYER_SPRITE_CROUCHING_HEIGHT_MULTIPLIER: f32 = 0.6;
const PLAYER_LOW_BLOCK_THRESHOLD_RATIO: f32 = 0.25;
const PLAYER_HIGH_BLOCK_THRESHOLD_RATIO: f32 = 0.75;

pub const PLAYER_SPRITE_CROUCHING_HEIGHT: f32 =
    PLAYER_SPRITE_STANDING_HEIGHT * PLAYER_SPRITE_CROUCHING_HEIGHT_MULTIPLIER;
pub const PLAYER_CROUCHING_OFFSET: f32 = PLAYER_SPRITE_STANDING_HEIGHT / 2.0;
pub const PLAYER_STANDING_OFFSET: f32 = PLAYER_SPRITE_CROUCHING_HEIGHT / 2.0;
pub const PLAYER_CROUCHING_SHIFT: f32 = PLAYER_STANDING_OFFSET - PLAYER_CROUCHING_OFFSET;
pub const PLAYER_STANDING_SHIFT: f32 = -PLAYER_CROUCHING_SHIFT;

pub struct PlayerStatePlugin;

impl Plugin for PlayerStatePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Inspectable, PartialEq, Clone, Copy, Debug, Default)]
pub struct MoveState {
    pub start_frame: usize,
    pub phase_index: usize,
    pub move_id: MoveId,
}
