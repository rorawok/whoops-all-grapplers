use bevy::prelude::*;
use input_parsing::InputParser;
use kits::{Kit, Resources};
use time::Clock;

const CHARGE_EXPIRATION_TIME: f32 = 0.2;
const CHARGE_EXPIRATION_FRAMES: usize = (CHARGE_EXPIRATION_TIME * constants::FPS) as usize;

pub fn manage_charge(mut query: Query<(&mut Resources, &InputParser, &Kit)>, clock: Res<Clock>) {
    for (mut resources, parser, kit) in query.iter_mut() {
        let charge = &mut resources.charge;

        let player_charging = kit
            .charge_directions
            .contains(&parser.get_relative_stick_position());

        if player_charging {
            // Bump charge
            charge.progress += 1;
            charge.last_update = clock.frame;
        } else if charge.last_update + CHARGE_EXPIRATION_FRAMES < clock.frame {
            // Charge expiration
            charge.reset();
        }
    }
}
