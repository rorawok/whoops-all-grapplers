mod assets;
mod camera;
mod character;
mod clock;
mod constants;
mod input;
mod labels;
mod player;

pub use assets::AssetsPlugin;
pub use camera::CameraPlugin;
pub use clock::ClockPlugin;
pub use labels::StagePlugin;
pub use player::PlayerPlugin;

pub(crate) use assets::Materials;
pub(crate) use clock::Clock;
pub(crate) use player::Player;
