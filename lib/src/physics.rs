use bevy::{prelude::*, sprite};
use bevy_inspector_egui::Inspectable;

use constants::PLAYER_GRAVITY_PER_FRAME;
use kits::{Kit, MoveId, MoveMobility};
use player_state::PlayerState;
use time::{once_per_combat_frame, WAGStage};
use types::{LRDirection, Player};

use crate::{
    camera::{WorldCamera, VIEWPORT_HALFWIDTH},
    spawner::Spawner,
};

pub const GROUND_PLANE_HEIGHT: f32 = 0.0;
pub const ARENA_WIDTH: f32 = 10.0;

#[derive(Debug, Default, Inspectable, Component)]
pub struct ConstantVelocity {
    pub shift: Vec3,
    pub speed: Vec3,
}
impl ConstantVelocity {
    pub fn new(speed: Vec3) -> ConstantVelocity {
        ConstantVelocity {
            speed,
            shift: speed / constants::FPS,
        }
    }
}

#[derive(Debug, Inspectable, Clone, Default, Copy)]
pub struct CurrentMove {
    id: MoveId,
    base_velocity: Vec3,
}
#[derive(Debug, Inspectable, Clone, Default, Copy, Component)]
pub struct PlayerVelocity {
    velocity: Vec3,
    current_move: Option<CurrentMove>,
}

impl PlayerVelocity {
    pub fn get_shift(&self) -> Vec3 {
        self.velocity / constants::FPS
    }
    pub fn add_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse;
    }
    pub fn drag(&mut self) {
        self.velocity = Vec3::new(
            if self.velocity.x.abs() > constants::DRAG {
                self.velocity.x.signum() * (self.velocity.x.abs() - constants::DRAG)
            } else {
                self.current_move = None;
                0.0
            },
            self.velocity.y,
            0.0,
        );
    }
    fn handle_move_velocity(
        &mut self,
        move_id: MoveId,
        mobility: MoveMobility,
        facing: &LRDirection,
    ) {
        match mobility {
            MoveMobility::Impulse(amount) => {
                self.handle_move_velocity_chaining(move_id, facing.mirror_vec(amount), false);
            }
            MoveMobility::Perpetual(amount) => {
                self.handle_move_velocity_chaining(move_id, facing.mirror_vec(amount), true);
            }
        }
    }
    fn handle_move_velocity_chaining(&mut self, id: MoveId, amount: Vec3, perpetual: bool) {
        let first_move = self.current_move.is_none();

        if first_move {
            // Move started
            self.velocity = amount;
            self.current_move = Some(CurrentMove {
                id,
                base_velocity: Vec3::ZERO,
            });
        } else {
            let current_move = self.current_move.unwrap();
            let move_continues = current_move.id == id;

            if move_continues {
                if perpetual {
                    // Continue perpetual motion
                    self.velocity = current_move.base_velocity + amount;
                }
            } else {
                // Cancel into a new move
                self.add_impulse(amount);

                self.current_move = Some(CurrentMove {
                    id,
                    base_velocity: self.velocity,
                });
            }
        }
    }

    fn handle_walking_velocity(&mut self, direction: LRDirection) {
        let proposed_walk_velocity =
            self.velocity.x + direction.mirror_f32(constants::PLAYER_ACCELERATION);

        self.velocity.x = direction.mirror_f32(
            proposed_walk_velocity
                .abs()
                .clamp(constants::MINIMUM_WALK_SPEED, constants::MAXIMUM_WALK_SPEED),
        );
        self.current_move = None;
    }

    fn handle_collisions(&mut self, clamped_position: &ClampedPosition) {
        if clamped_position.touching_wall() {
            self.x_collision();
        }
        if clamped_position.touching_floor {
            self.y_collision();
        }
    }

    fn x_collision(&mut self) {
        // Just stop for now, but can be used to implement bounces and whatnot in the future
        self.velocity.x = 0.0;
    }

    fn y_collision(&mut self) {
        // Hit the floor
        self.velocity.y = 0.0;
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            WAGStage::Physics,
            SystemSet::new()
                .with_run_criteria(once_per_combat_frame)
                .with_system(sideswitcher)
                .with_system(player_gravity.after(sideswitcher))
                .with_system(player_input.after(player_gravity))
                .with_system(move_constants.after(player_input))
                .with_system(move_players.after(move_constants)),
        );
    }
}

fn player_input(mut query: Query<(&PlayerState, &mut PlayerVelocity, &Kit, &LRDirection)>) {
    for (state, mut velocity, kit, facing) in query.iter_mut() {
        if let Some(Some((move_id, mobility))) = state.get_move_state().map(|move_state| {
            kit.get_move(move_state.move_id)
                .get_action(move_state)
                .unwrap()
                .0
                .get_mobility()
                .map(|mobility| (move_state.move_id, mobility))
        }) {
            velocity.handle_move_velocity(move_id, mobility, facing);
        } else if let Some(walk_direction) = state.get_walk_direction() {
            velocity.handle_walking_velocity(walk_direction);
        } else if state.is_grounded() {
            velocity.drag();
        }
    }
}

fn sideswitcher(
    mut players: Query<(Entity, &Transform, &mut LRDirection), With<Player>>,
    others: Query<(Entity, &Transform), With<Player>>,
) {
    for (entity, transform, mut facing) in players.iter_mut() {
        for (e, tf) in others.iter() {
            if e == entity {
                continue;
            }

            facing.set_flipped(transform.translation.x > tf.translation.x);
        }
    }
}

fn move_constants(
    mut commands: Commands,
    mut query: Query<(Entity, &ConstantVelocity, &mut Transform)>,
) {
    // Handle static collision
    for (entity, velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.shift;

        // Despawn the thing if it's outside of the arena
        if transform.translation.length() > ARENA_WIDTH + 10.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// TODO: This could use a worldquery
#[allow(clippy::type_complexity)]
fn move_players(
    mut queries: ParamSet<(
        Query<(&mut PlayerVelocity, &mut Transform, &PlayerState, &Kit)>,
        Query<&Transform, With<WorldCamera>>,
    )>,
) {
    let arena_rect = legal_position_space(queries.p1().single().translation.x);

    let mut player_query = queries.p0();
    if let Some([(mut velocity1, mut tf1, state1, kit1), (mut velocity2, mut tf2, state2, kit2)]) =
        player_query.iter_combinations_mut().fetch_next()
    {
        let collider_size1 = kit1.get_size(state1.is_crouching());
        let collider_size2 = kit2.get_size(state2.is_crouching());

        let clamped_position1 = clamp_position(
            tf1.translation + velocity1.get_shift(),
            collider_size1,
            arena_rect,
        );
        let clamped_position2 = clamp_position(
            tf2.translation + velocity2.get_shift(),
            collider_size2,
            arena_rect,
        );

        tf1.translation = clamped_position1.position;
        tf2.translation = clamped_position2.position;

        velocity1.handle_collisions(&clamped_position1);
        velocity2.handle_collisions(&clamped_position2);

        if let Some(push_force) = push_force(
            clamped_position1.position,
            collider_size1,
            clamped_position2.position,
            collider_size2,
        ) {
            let can_move1 = clamped_position1.can_move_horizontally(push_force);
            let can_move2 = clamped_position2.can_move_horizontally(-push_force);
            assert!(
                can_move1 || can_move2,
                "Both players are blocked by walls somehow"
            );

            if can_move1 && can_move2 {
                // Both can move
                tf1.translation += Vec3::X * push_force / 2.0;
                tf2.translation -= Vec3::X * push_force / 2.0;
            } else if can_move1 {
                // 1 can move, 2 cannot
                velocity1.x_collision();
                tf1.translation += Vec3::X * push_force;
            } else {
                // 2 can move, 1 cannot
                velocity2.x_collision();
                tf2.translation -= Vec3::X * push_force;
            }
        }
    }
}

fn player_gravity(
    mut commands: Commands,
    mut players: Query<(
        &mut PlayerVelocity,
        &mut PlayerState,
        &mut Spawner,
        &Transform,
    )>,
) {
    for (mut velocity, mut state, mut spawner, tf) in players.iter_mut() {
        let is_airborne = tf.translation.y > GROUND_PLANE_HEIGHT;

        if is_airborne {
            velocity.add_impulse(-Vec3::Y * PLAYER_GRAVITY_PER_FRAME);
            if state.is_grounded() {
                state.jump();
            }
        } else if !state.is_grounded() {
            state.land();
            spawner.despawn_on_phase_change(&mut commands);
        }
    }
}

#[derive(Debug)]
struct ClampedPosition {
    position: Vec3,
    touching_right_wall: bool,
    touching_left_wall: bool,
    touching_floor: bool,
}
impl ClampedPosition {
    fn touching_wall(&self) -> bool {
        self.touching_left_wall || self.touching_right_wall
    }

    fn can_move_horizontally(&self, amount: f32) -> bool {
        if amount > 0.0 {
            // Moving right
            !self.touching_right_wall
        } else {
            // Moving left
            !self.touching_left_wall
        }
    }
}

fn clamp_position(position: Vec3, size: Vec2, arena_rect: Rect<f32>) -> ClampedPosition {
    let halfsize = size / 2.0;

    let player_rect = Rect {
        left: position.x - halfsize.x,
        right: position.x + halfsize.x,
        bottom: position.y,
        top: position.y + size.y,
    };

    let touching_right_wall = player_rect.right >= arena_rect.right;
    let touching_left_wall = player_rect.left <= arena_rect.left;
    let clamped_x = if touching_right_wall {
        arena_rect.right - halfsize.x
    } else if touching_left_wall {
        arena_rect.left + halfsize.x
    } else {
        position.x
    };

    let touching_floor = player_rect.bottom <= arena_rect.bottom;
    let clamped_y = if touching_floor {
        arena_rect.bottom
    } else {
        position.y
    };

    ClampedPosition {
        position: Vec3::new(clamped_x, clamped_y, 0.0),
        touching_right_wall,
        touching_left_wall,
        touching_floor,
    }
}

const CAMERA_EDGE_COLLISION_PADDING: f32 = 0.5;
fn legal_position_space(camera_x: f32) -> Rect<f32> {
    // Camera x is clamped in camera moving system, so that camera_x + VIEWPORT_HALFWIDTH = ARENA_WIDTH
    Rect {
        bottom: GROUND_PLANE_HEIGHT,
        right: camera_x + VIEWPORT_HALFWIDTH - CAMERA_EDGE_COLLISION_PADDING,
        left: camera_x - VIEWPORT_HALFWIDTH + CAMERA_EDGE_COLLISION_PADDING,
        top: std::f32::INFINITY,
    }
}

pub fn vec_rect_collision(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    // Bevy collide only detects collisions if the edges overlap, most of the time this is good enough
    // But occasionally a collider spawns inside another, in which case we need a check for that.
    let a = sprite::Rect {
        min: a_pos.truncate() - a_size / 2.0,
        max: a_pos.truncate() + a_size / 2.0,
    };
    let b = sprite::Rect {
        min: b_pos.truncate() - b_size / 2.0,
        max: b_pos.truncate() + b_size / 2.0,
    };

    rect_collision(a, b)
}

pub fn rect_collision(a: sprite::Rect, b: sprite::Rect) -> bool {
    let x_overlap = a.min.x < b.max.x && a.max.x > b.min.x;
    let y_overlap = a.min.y < b.max.y && a.max.y > b.min.y;
    x_overlap && y_overlap
}

fn push_force(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<f32> {
    if vec_rect_collision(a_pos, a_size, b_pos, b_size) {
        let clean_distance = (a_size + b_size).x / 2.0;
        let distance = (a_pos - b_pos).x;
        Some(distance.signum() * ((clean_distance / distance.abs()) - 1.0))
    } else {
        None
    }
}
