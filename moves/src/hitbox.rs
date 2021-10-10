use bevy::prelude::*;

use bevy::utils::HashMap;
use types::{MoveType, Player};

use crate::ryan::*;

#[derive(Default, Clone, Copy)]
pub struct Hitbox {
    offset: Vec3,
    pub size: Vec2,
    pub on_hit_damage: Option<f32>,
    pub owner: Option<Player>,
}
impl Hitbox {
    pub fn get_offset(&self, flipped: bool) -> Vec3 {
        if flipped {
            Vec3::new(-self.offset.x, self.offset.y, self.offset.z)
        } else {
            self.offset
        }
    }

    pub fn new(offset: Vec2, size: Vec2, damage: Option<f32>) -> Self {
        Self {
            offset: offset.extend(0.0),
            size,
            on_hit_damage: damage,
            owner: None,
        }
    }
}

pub fn ryan_hitboxes() -> HashMap<MoveType, Hitbox> {
    vec![
        (
            HADOUKEN,
            Hitbox::new(Vec2::new(0.5, 0.5), Vec2::new(0.3, 0.2), Some(0.3)),
        ),
        (
            PUNCH,
            Hitbox::new(Vec2::new(1.0, 0.5), Vec2::new(0.2, 0.3), Some(0.2)),
        ),
        (
            COMMAND_PUNCH,
            Hitbox::new(Vec2::new(0.5, 0.5), Vec2::new(1.0, 1.0), Some(0.4)),
        ),
    ]
    .into_iter()
    .collect()
}
