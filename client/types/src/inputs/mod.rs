mod stick_position;
use bevy_inspector_egui::Inspectable;
pub use stick_position::StickPosition;

use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Inspectable, Default)]
/// Buttons of the game
/// The name 'Button' is in prelude
pub enum GameButton {
    #[default]
    Default, // To satisfy Inspectable

    Grab,
    Strong,
    Fast,
    Equipment,
    Taunt,
}
