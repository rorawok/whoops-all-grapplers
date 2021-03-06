use std::collections::HashSet;

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use types::{Animation, GameButton};

use crate::{Cost, ItemId};

use super::{MoveAction, MoveId, MoveSituation, MoveType, Phase};

#[derive(Debug, Default, Inspectable, Clone, PartialEq)]
pub struct Move {
    pub input: Option<&'static str>,
    pub animation: Animation,
    pub move_type: MoveType,
    pub phases: Vec<Branch>,
    pub requirements: Requirements,
}

impl Move {
    pub fn get_action_index(&self, situation: &MoveSituation, current_frame: i32) -> Option<usize> {
        // Can be negative, which is why cast before operation
        let mut frames_left = current_frame - situation.start_frame;

        for (index, phase) in self
            .phases
            .iter()
            .map(|resolver| resolver.get(situation).0)
            .enumerate()
        {
            if let Some(duration) = phase.get_duration() {
                frames_left -= duration as i32;
                if frames_left < 0 {
                    return Some(index);
                }
            } else {
                // Current instruction is a move, it should be returned despite the time.
                return Some(index);
            }
        }
        None
    }

    pub fn get_action(
        &self,
        situation: &MoveSituation,
    ) -> Option<(MoveAction, Option<Requirements>)> {
        let switch = self.phases.get(situation.phase_index)?.to_owned();

        Some(switch.get(situation))
    }

    pub fn get_animation(&self, situation: &MoveSituation) -> Animation {
        if let Some(action) = self.get_action(situation) {
            if let MoveAction::Phase(phase) = action.0 {
                if let Some(phase_animation) = phase.animation {
                    return phase_animation;
                }
            }
        }
        self.animation
    }
}

#[derive(Debug, Inspectable, Clone, PartialEq, Default)]
pub struct Branch {
    pub default: MoveAction,
    pub branches: Vec<(Requirements, MoveAction)>, // This way order is maintained
}
impl Branch {
    pub fn get(&self, situation: &MoveSituation) -> (MoveAction, Option<Requirements>) {
        for (requirements, phase) in &self.branches {
            if situation.fulfills(requirements, None) {
                return (phase.to_owned(), Some(requirements.to_owned()));
            }
        }
        (self.default.to_owned(), None)
    }
}
impl From<Phase> for Branch {
    fn from(phase: Phase) -> Self {
        Branch {
            default: phase.into(),
            branches: vec![],
        }
    }
}
impl From<MoveId> for Branch {
    fn from(id: MoveId) -> Self {
        Branch {
            default: id.into(),
            branches: vec![],
        }
    }
}

#[derive(Debug, Default, Inspectable, Clone, Eq, PartialEq)]
pub struct Requirements {
    pub has_hit: Option<bool>,
    pub cost: Option<Cost>,
    #[inspectable(ignore)]
    pub items: Option<HashSet<ItemId>>,
    #[inspectable(ignore)]
    pub buttons_held: Option<HashSet<GameButton>>,
    pub grounded: Option<bool>,
}
impl Requirements {
    pub fn has_hit() -> Requirements {
        Requirements {
            has_hit: Some(true),
            ..default()
        }
    }
}
