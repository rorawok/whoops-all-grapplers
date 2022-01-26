use bevy_inspector_egui::Inspectable;
use types::{LRDirection, MoveId};

#[derive(Inspectable, PartialEq, Clone, Copy, Debug)]
pub enum GroundActivity {
    Stun(usize),
    Move(MoveId),
    Walk(usize, LRDirection),
    PreJump {
        launch_frame: usize,
        direction: Option<LRDirection>,
        held: bool,
    },
    Crouching,
    Standing,
}

impl Default for GroundActivity {
    fn default() -> Self {
        GroundActivity::Standing
    }
}
