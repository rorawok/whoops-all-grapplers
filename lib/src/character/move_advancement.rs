use bevy::prelude::*;
use input_parsing::InputParser;
use moves::{MoveBank, PhaseKind};
use player_state::PlayerState;
use time::Clock;
use types::{Grabable, LRDirection, Player};

use crate::{assets::Colors, damage::Health, physics::PlayerVelocity, spawner::Spawner};

#[allow(clippy::type_complexity)]
pub fn move_advancement(
    mut commands: Commands,
    colors: Res<Colors>,
    clock: Res<Clock>,
    mut players: Query<(
        &mut PlayerState,
        &mut Spawner,
        &MoveBank,
        Entity,
        &LRDirection,
        &Player,
        &Transform,
        &Grabable,
        &InputParser,
        &mut PlayerVelocity,
        &mut Health,
    )>,
) {
    let mut iter = players.iter_combinations_mut();
    if let Some([mut p1, mut p2]) = iter.fetch_next() {
        advance_move(&mut commands, &clock, &colors, &mut p1, &mut p2);
        advance_move(&mut commands, &clock, &colors, &mut p2, &mut p1);
    }
}

type ComponentList<'a> = (
    Mut<'a, PlayerState>,
    Mut<'a, Spawner>,
    &'a MoveBank,
    Entity,
    &'a LRDirection,
    &'a Player,
    &'a Transform,
    &'a Grabable,
    &'a InputParser,
    Mut<'a, PlayerVelocity>,
    Mut<'a, Health>,
);

fn advance_move(
    commands: &mut Commands,
    clock: &Clock,
    colors: &Res<Colors>,
    actor: &mut ComponentList,
    target: &mut ComponentList,
) {
    let (state1, spawner1, bank, parent, facing, player, tf1, _, _, _, _) = actor;
    let (state2, spawner2, _, _, _, _, tf2, grab_target, parser, velocity, health) = target;

    if let Some(move_state) = state1.get_move_state() {
        let move_data = bank.get(move_state.move_id);
        if let Some(phase_index) = move_data.get_phase_index(move_state.start_frame, clock.frame) {
            if move_state.phase_index != phase_index {
                // Despawn old things
                spawner1.despawn_on_phase_change(commands);

                match move_data.get_phase(phase_index).kind {
                    PhaseKind::Attack(descriptor) => spawner1.spawn_attack(
                        move_state.move_id,
                        descriptor,
                        commands,
                        colors,
                        clock.frame,
                        *parent,
                        facing,
                        **player,
                        tf1.translation,
                    ),
                    PhaseKind::Grab(descriptor) => {
                        let grab_origin = tf1.translation + descriptor.offset.extend(0.0);
                        let distance = (grab_origin - tf2.translation).length();
                        let max_distance = grab_target.size + descriptor.range;
                        let in_range = distance <= max_distance;

                        let teched = state2.get_move_state().is_none() && parser.clear_head();

                        if in_range && !teched {
                            state2.throw();
                            spawner2.despawn_on_hit(commands);
                            velocity.add_impulse(descriptor.impulse);
                            health.apply_damage(descriptor.damage);
                        }
                    }
                    PhaseKind::Animation => {}
                };
                // Start next phase
                state1.set_move_phase_index(phase_index);
            }
        } else {
            // Move has ended
            spawner1.despawn_on_phase_change(commands);
            state1.recover();
        }
    }
}
