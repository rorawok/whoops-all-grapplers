use bevy::prelude::*;

use input_parsing::InputParser;
use items::Inventory;
use moves::{CancelLevel, Move, MoveBank, MoveStartCondition, MoveState};
use player_state::PlayerState;
use time::Clock;
use types::MoveId;

use crate::{
    resources::{Charge, GameResource, Meter},
    spawner::Spawner,
};
const EVENT_REPEAT_PERIOD: f32 = 0.3; // In seconds
const FRAMES_TO_LIVE_IN_BUFFER: usize = (EVENT_REPEAT_PERIOD * constants::FPS) as usize;

#[derive(Debug, Default, Component)]
pub struct MoveBuffer {
    buffer: Vec<(usize, MoveId)>,
    force_start: Option<(MoveId, Move)>,
}
impl MoveBuffer {
    pub fn set_force_starter(&mut self, move_id: MoveId, move_data: Move) {
        self.force_start = Some((move_id, move_data));
    }

    fn add_events(&mut self, events: Vec<MoveId>, frame: usize) {
        self.buffer.extend(events.into_iter().map(|id| (frame, id)));
    }

    fn use_move(
        &mut self,
        bank: &MoveBank,
        active_move: Option<MoveState>,
        grounded: bool,
        meter: &Meter,
        charge: Option<&Charge>,
    ) -> Option<(MoveId, Move)> {
        if self.force_start.is_some() {
            // Early return for the cases when a move has forked
            return self.force_start.take();
        }

        let cancel_requirement = if let Some(move_state) = active_move {
            let move_data = bank.get(move_state.move_id);
            if move_data.get_action(move_state).is_cancellable() {
                move_data.cancel_level
            } else {
                CancelLevel::Uncancellable
            }
        } else {
            CancelLevel::Anything
        };

        if let Some((selected_id, move_data)) = self
            .buffer
            .iter()
            .map(|(_, id)| (*id, bank.get(*id).to_owned()))
            .filter(|(_, move_data)| {
                if grounded {
                    move_data.conditions.contains(MoveStartCondition::GROUND)
                } else {
                    move_data.conditions.contains(MoveStartCondition::AIR)
                }
            })
            .filter(|(_, action)| action.cancel_level > cancel_requirement)
            .filter(|(_, action)| meter.can_afford(action.cost.meter))
            .filter(|(_, action)| {
                !action.cost.charge // Either move doesn't require charge
                    || (charge.is_some() // Or the player can afford it
                        && charge.unwrap().can_afford(action.cost.charge))
            })
            .min_by(|(id1, _), (id2, _)| id1.cmp(id2))
        {
            self.buffer.retain(|(_, id)| selected_id != *id);
            Some((selected_id, move_data))
        } else {
            None
        }
    }

    fn clear_old(&mut self, current_frame: usize) {
        self.buffer.retain(|(frame, _)| {
            if current_frame > *frame {
                // Default case, retain those who are fresh
                current_frame - frame < FRAMES_TO_LIVE_IN_BUFFER
            } else {
                // Round has restarted, clear the buffer
                false
            }
        });
    }

    pub fn clear(&mut self) {
        *self = MoveBuffer::default();
    }
}

#[allow(clippy::type_complexity)]
pub fn move_activator(
    mut commands: Commands,
    clock: Res<Clock>,
    mut query: Query<(
        &mut InputParser,
        &mut PlayerState,
        &mut MoveBuffer,
        &MoveBank,
        &mut Meter,
        Option<&mut Charge>,
        &mut Spawner,
        &Inventory,
    )>,
) {
    for (
        mut reader,
        mut state,
        mut buffer,
        bank,
        mut meter,
        mut maybe_charge,
        mut spawner,
        inventory,
    ) in query.iter_mut()
    {
        buffer.clear_old(clock.frame);
        buffer.add_events(reader.drain_events(), clock.frame);

        if state.stunned() {
            continue;
        }

        if let Some((move_id, move_data)) = buffer.use_move(
            bank,
            state.get_move_state(),
            state.is_grounded(),
            &meter,
            maybe_charge.as_deref(),
        ) {
            spawner.despawn_on_phase_change(&mut commands);
            state.start_move(move_id, clock.frame, inventory.phase_flags());
            meter.pay(move_data.cost.meter);

            if let Some(ref mut charge) = maybe_charge {
                charge.pay(move_data.cost.charge);
            }
        }
    }
}
