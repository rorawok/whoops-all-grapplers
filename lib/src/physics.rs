use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use player_state::{PlayerState, StateEvent};
use types::{LRDirection, Player};

use crate::{
    camera::{WorldCamera, VIEWPORT_WIDTH},
    clock::run_max_once_per_combat_frame,
};

pub const GROUND_PLANE_HEIGHT: f32 = -0.4;
pub const ARENA_WIDTH: f32 = 10.0;

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

#[derive(Debug, Inspectable, Clone)]
enum PlayerVelocityType {
    Walk,
    Move,
    Previous,
}

#[derive(Debug, Inspectable, Clone)]
pub struct PlayerVelocity {
    total: Vec3,
    walk_velocity: f32,
    dash_velocity: Vec3,
    impulse_collector: Vec3,
    used_velocity: PlayerVelocityType,
}

impl Default for PlayerVelocity {
    fn default() -> Self {
        Self {
            total: Default::default(),
            walk_velocity: Default::default(),
            dash_velocity: Default::default(),
            impulse_collector: Default::default(),
            used_velocity: PlayerVelocityType::Previous,
        }
    }
}
impl PlayerVelocity {
    pub fn get_total(&self) -> Vec3 {
        match self.used_velocity {
            PlayerVelocityType::Walk => Vec3::X * self.walk_velocity,
            PlayerVelocityType::Move => self.dash_velocity,
            PlayerVelocityType::Previous => self.total,
        }
    }
    pub fn get_shift(&self) -> Vec3 {
        self.get_total() / constants::FPS
    }
    pub fn add_impulse(&mut self, impulse: Vec3) {
        self.impulse_collector += impulse;
    }
    pub fn tick(&mut self, state: &mut PlayerState) {
        // Set correct velocity mode
        if state.get_move_mobility().is_some() {
            self.used_velocity = PlayerVelocityType::Move;
        } else if state.get_walk_direction().is_some() {
            self.used_velocity = PlayerVelocityType::Walk;
        } else {
            self.used_velocity = PlayerVelocityType::Previous;
        }

        // Calculate new velocity
        self.total = match self.used_velocity {
            PlayerVelocityType::Walk => Vec3::new(self.walk_velocity, 0.0, 0.0),
            PlayerVelocityType::Move => self.dash_velocity,
            PlayerVelocityType::Previous => {
                if state.is_grounded() {
                    // Drag
                    Vec3::new(
                        if self.total.length() > constants::DRAG {
                            self.total.x.signum() * (self.total.x.abs() - constants::DRAG)
                        } else {
                            0.0
                        },
                        self.total.y,
                        0.0,
                    )
                } else {
                    // Gravity
                    Vec3::new(
                        self.total.x,
                        self.total.y - constants::PLAYER_GRAVITY_PER_FRAME,
                        0.0,
                    )
                }
            }
        } + self.impulse_collector;
        self.impulse_collector = Vec3::ZERO;
    }
    fn set_move_velocity(&mut self, state: &mut PlayerState, facing: &LRDirection) {
        self.dash_velocity = state
            .get_move_mobility()
            .map(|mobility| facing.mirror_vec(mobility))
            .unwrap_or(Vec3::ZERO);
    }

    fn set_walking_velocity(&mut self, direction: Option<LRDirection>) {
        if let Some(direction) = direction {
            let acceleration = direction.mirror_f32(constants::PLAYER_ACCELERATION);

            if self.walk_velocity.abs() < constants::MINIMUM_WALK_SPEED
                || (self.walk_velocity.signum() - acceleration.signum()).abs() < 1.1
            {
                // If player is starting to move, or keeps moving in the same direction
                let proposed_walk_velocity = self.walk_velocity + acceleration;

                self.walk_velocity = proposed_walk_velocity.signum()
                    * proposed_walk_velocity
                        .abs()
                        .clamp(constants::MINIMUM_WALK_SPEED, constants::MAXIMUM_WALK_SPEED)
            } else {
                self.walk_velocity = 0.0;
            }
        } else {
            self.walk_velocity = 0.0;
        }
    }

    fn x_collision(&mut self) {
        // Just stop for now, but can be used to implement bounces and whatnot in the future
        self.total.x = 0.0;
    }

    fn y_collision(&mut self) {
        // Hit the floor
        self.total.y = 0.0;
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_max_once_per_combat_frame.system())
                .with_system(player_input.system())
                .with_system(sideswitcher.system())
                .with_system(push_players.system())
                .with_system(move_players.system())
                .with_system(move_constants.system()),
        );
    }
}

fn player_input(mut query: Query<(&mut PlayerState, &mut PlayerVelocity, &LRDirection)>) {
    for (mut state, mut velocity, facing) in query.iter_mut() {
        velocity.set_move_velocity(&mut state, facing);
        for event in state.get_events() {
            match event {
                StateEvent::Jump(impulse) => {
                    velocity.add_impulse(impulse);
                    state.consume_event(event);
                }
                StateEvent::Null => panic!("Null event from player state"),
                _ => {}
            }
        }
        velocity.set_walking_velocity(state.get_walk_direction());
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
        if transform.translation.length() > ARENA_WIDTH + 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
fn move_players(
    mut queries: QuerySet<(
        Query<(&mut PlayerVelocity, &mut Transform, &mut PlayerState)>,
        Query<&Transform, With<WorldCamera>>,
    )>,
) {
    let camera_x = queries
        .q1()
        .single()
        .map(|camtf| camtf.translation.x)
        .unwrap_or_default();

    // Handle static collision
    for (mut velocity, mut transform, mut state) in queries.q0_mut().iter_mut() {
        velocity.tick(&mut state);

        let shift = velocity.get_shift();

        if let Some(collision) = static_collision(
            transform.translation,
            shift,
            state.get_collider_size(),
            camera_x,
        ) {
            transform.translation = collision.legal_position;
            if collision.x_collision {
                velocity.x_collision();
            }

            if collision.y_collision {
                velocity.y_collision();
                state.land()
            }
        } else {
            transform.translation += shift;
        }
    }
}

#[allow(clippy::type_complexity)]
fn push_players(
    players: Query<Entity, With<Player>>,
    mut query_set: QuerySet<(
        Query<(&PlayerVelocity, &Transform, &PlayerState, &LRDirection)>,
        Query<&mut PlayerVelocity>,
    )>,
) {
    for entity1 in players.iter() {
        for entity2 in players.iter() {
            if entity1 != entity2 {
                let (velocity1, transform1, player1, facing1) =
                    query_set.q0().get(entity1).unwrap();
                let (velocity2, transform2, player2, _) = query_set.q0().get(entity2).unwrap();

                let future_position1 = transform1.translation + velocity1.get_shift();
                let future_position2 = transform2.translation + velocity2.get_shift();

                if rect_collision(
                    future_position1,
                    player1.get_collider_size(),
                    future_position2,
                    player2.get_collider_size(),
                ) {
                    // Player-player collision is happening
                    let distance = (transform1.translation - transform2.translation).length();

                    if distance > constants::PUSHING_DEAD_ZONE {
                        let moving_closer =
                            (future_position1 - future_position2).length() < distance;

                        // Don't push when really close, this is to prevent spazzing as directions change
                        let push_vector = Vec3::new(
                            constants::PUSHING_IMPULSE
                                * if moving_closer {
                                    // Go backwards
                                    -facing1.to_signum()
                                } else {
                                    // Go to current direction
                                    let val = velocity1.get_total().x;
                                    if val == 0.0 {
                                        val
                                    } else {
                                        val.signum()
                                    }
                                },
                            0.0,
                            0.0,
                        );

                        let mut object = query_set.q1_mut().get_mut(entity1).unwrap();
                        object.add_impulse(push_vector);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct StaticCollision {
    legal_position: Vec3, // How much space there is to move
    x_collision: bool,
    y_collision: bool,
}
impl StaticCollision {
    fn did_collide(&self) -> bool {
        self.x_collision || self.y_collision
    }

    fn wrap(self) -> Option<StaticCollision> {
        if self.did_collide() {
            Some(self)
        } else {
            None
        }
    }
}

const CAMERA_EDGE_COLLISION_PADDING: f32 = 1.0;

fn static_collision(
    current_position: Vec3,
    movement: Vec3,
    player_size: Vec2,
    camera_x: f32,
) -> Option<StaticCollision> {
    let future_position = current_position + movement;
    let relative_ground_plane = GROUND_PLANE_HEIGHT + player_size.y / 2.0;

    let distance_to_ground = future_position.y - relative_ground_plane;
    let y_collision = distance_to_ground < 0.0;
    let legal_y = if y_collision {
        relative_ground_plane
    } else {
        future_position.y
    };

    let right_wall = ARENA_WIDTH.min(camera_x + VIEWPORT_WIDTH - CAMERA_EDGE_COLLISION_PADDING);
    let left_wall = (-ARENA_WIDTH).max(camera_x - VIEWPORT_WIDTH + CAMERA_EDGE_COLLISION_PADDING);

    let (legal_x, x_collision) = if future_position.x > right_wall {
        (right_wall, true)
    } else if future_position.x < left_wall {
        (left_wall, true)
    } else {
        (future_position.x, false)
    };

    StaticCollision {
        legal_position: Vec3::new(legal_x, legal_y, 0.0),
        x_collision,
        y_collision,
    }
    .wrap()
}

pub fn rect_collision(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    // Bevy collide only detects collisions if the edges overlap, most of the time this is good enough
    // But occasionally a collider spawns inside another, in which case we need a check for that.
    let a_min = a_pos.truncate() - (a_size / 2.0);
    let a_max = a_pos.truncate() + (a_size / 2.0);
    let b_min = b_pos.truncate() - (b_size / 2.0);
    let b_max = b_pos.truncate() + (b_size / 2.0);

    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        return true;
    }
    false
}
