use bevy::{prelude::*, utils::HashMap};
use types::Player;

use crate::{clock::Clock, damage::Health};

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum GameState {
    Combat,
    PreRound,
    PostRound,
}

struct InterFrameCountdown(Timer);

pub struct RoundResult {
    pub winner: Option<Player>,
}

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Combat).with_system(check_dead))
            .add_system_set(
                SystemSet::on_enter(GameState::PostRound).with_system(restart_countdown),
            )
            .add_system_set(SystemSet::on_update(GameState::PostRound).with_system(tick_countdown))
            .add_system_set(SystemSet::on_update(GameState::PreRound).with_system(tick_countdown))
            .add_state(GameState::PreRound)
            .insert_resource(InterFrameCountdown(Timer::from_seconds(3.0, false)));
    }
}

const ALMOST_ZERO: f32 = 0.0001; // 0.01% of hp, this is used to get around rounding errors
fn check_dead(
    mut commands: Commands,
    clock: Res<Clock>,
    query: Query<(&Health, &Player)>,
    mut state: ResMut<State<GameState>>,
) {
    if query
        .iter()
        .any(|(health, _)| health.get_ratio() <= ALMOST_ZERO)
        || clock.time_out()
    {
        let healths: HashMap<&Player, &Health> = query.iter().map(|(h, p)| (p, h)).collect();

        commands.insert_resource(
            if healths.get(&Player::One).unwrap().get_ratio()
                - healths.get(&Player::Two).unwrap().get_ratio()
                > ALMOST_ZERO
            {
                RoundResult {
                    winner: healths
                        .into_iter()
                        .max_by(|(_, h1), (_, h2)| {
                            h1.get_ratio().partial_cmp(&h2.get_ratio()).unwrap()
                        })
                        .map(|(p, _)| *p),
                }
            } else {
                RoundResult { winner: None }
            },
        );

        state.set(GameState::PostRound).unwrap();
    }
}

fn restart_countdown(mut countdown: ResMut<InterFrameCountdown>) {
    countdown.0.reset();
}

fn tick_countdown(
    mut countdown: ResMut<InterFrameCountdown>,
    time: Res<Time>,
    mut state: ResMut<State<GameState>>,
) {
    countdown.0.tick(time.delta());
    if countdown.0.finished() {
        state.set(GameState::Combat).unwrap();
    }
}
