
use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::{prelude::*, render::pass::ClearColor};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;
use bevy_rapier2d::physics::{
    ColliderHandleComponent, RapierPhysicsPlugin, RigidBodyHandleComponent,
};
use bevy_rapier2d::rapier::dynamics::{RigidBodyBuilder, RigidBodySet};
use bevy_rapier2d::rapier::geometry::{ColliderBuilder, ColliderSet};
use bevy_rapier2d::rapier::math::Isometry;
use bevy_rapier2d::render::RapierRenderPlugin;

use crate::teams::*;
use crate::physics::*;
use crate::*;

/// stores units that are within attack (melee or missle) range of the
/// associated unit, used to determining unit behaviour.
#[derive(Debug, Default)]
pub struct NearbyUnitsComponent {
    melee_range: Vec<Entity>,
    missle_range: Vec<Entity>,
}

/// handles events that changes the commands and state for each unit.
/// processes the following inputs:
/// - unit proximity interactions
/// - TODO user commands
pub fn unit_event_system(
    game_speed: Res<GameSpeed>,
    mut state: Local<UnitInteractionState>,
    events: Res<Events<UnitInteractionEvent>>,
    units: Query<&mut UnitComponent>,
    nearbys: Query<&mut NearbyUnitsComponent>,
) {
    if game_speed.is_paused() { return; }

    /// this processes interactions one unit at a time within its own scope
    /// so we don't double-borrow the Unit Component
    fn process_unit_proximity(
        unit_id: Entity,
        target_id: Entity,
        nearbys: &Query<&mut NearbyUnitsComponent>,
        e_or_e: EnterOrExit,
        range_type: AttackType,
    ) {
        let mut nbs = nearbys.get_mut::<NearbyUnitsComponent>(unit_id).unwrap();
        let vec = if range_type == AttackType::Melee {
            &mut nbs.melee_range
        } else {
            &mut nbs.missle_range
        };

        if e_or_e == EnterOrExit::Enter {
            vec.push(target_id);
        } else {
            vec.retain(|e| e != &target_id);
        }
    }

    fn process_unit_command(
        unit_id: Entity,
        cmd: UnitUiCommand,
        units: &Query<&mut UnitComponent>,
    ) {
        use UnitUiCommand::*;
        let mut unit = units.get_mut::<UnitComponent>(unit_id).unwrap();
        match cmd {
            Attack(target, speed) => {
                unit.is_running = speed == UnitUiSpeedCommand::Run;
                unit.current_command = if let AttackType::Melee = unit.primary_attack_type() {
                    UnitUserCommand::AttackMelee(target)
                } else {
                    UnitUserCommand::AttackMissile(target)
                }
            }
            Move(pos, speed) => {
                unit.current_command = UnitUserCommand::Move(pos);
                unit.is_running = speed == UnitUiSpeedCommand::Run;
            }
            Stop => {
                unit.current_command = UnitUserCommand::None_;
            }
            ToggleGuardMode => unit.guard_mode_enabled = !unit.guard_mode_enabled,
            ToggleFireAtWill => unit.fire_at_will = !unit.fire_at_will,
            ToggleSpeed => unit.is_running = !unit.is_running,
        }
        log::debug!("unit current command: {:?}", unit.current_command);
    }
    
    // process state updates for units that have new events
    for event in state.event_reader.iter(&events) {
        log::debug!("event: {:?}", &event);
        match event.clone() {
            UnitInteractionEvent::Proximity(contact) => {
                match contact {
                    ContactType::UnitFiringRangeEnter{range_of, target} => {
                        process_unit_proximity(range_of, target, &nearbys, contact.enter_or_exit(), AttackType::Ranged);
                    }   
                    ContactType::UnitFiringRangeExit{range_of, target } => {
                        process_unit_proximity(range_of, target, &nearbys, contact.enter_or_exit(), AttackType::Ranged);
                    }   
                    ContactType::UnitUnitMeleeExit(e1, e2) => {
                        // separate scopes so we don't double-borrow the Unit component
                        process_unit_proximity(e1, e2, &nearbys, contact.enter_or_exit(), AttackType::Melee);
                        process_unit_proximity(e2, e1, &nearbys, contact.enter_or_exit(), AttackType::Melee);
                    }
                    ContactType::UnitUnitMeleeEnter(e1, e2) => {
                        process_unit_proximity(e1, e2, &nearbys, contact.enter_or_exit(), AttackType::Melee);
                        process_unit_proximity(e2, e1, &nearbys, contact.enter_or_exit(), AttackType::Melee);
                    }
                    ContactType::UnitWaypointReached(e1) => {
                        let mut unit = units.get_mut::<UnitComponent>(e1).unwrap();
                        unit.current_command = UnitUserCommand::None_;
                    }
                }             
            }
            UnitInteractionEvent::Ui(entity, cmd) => {
                process_unit_command(entity, cmd, &units);
            }
        }
    }
}


/// Returns None if no targets available
/// TODO make this fancier
pub fn pick_missile_target(
    available_targets: &Vec<Entity>
) -> Option<Entity> {
    available_targets.get(0).map(|e| e.clone())
}

/// Returns None if no targets available
/// TODO make this fancier
pub fn pick_melee_target(
    available_targets: &Vec<Entity>
) -> Option<Entity> {
    available_targets.get(0).map(|e| e.clone())
}

pub fn calculate_next_unit_state(
    current_command: &UnitUserCommand,
    enemies_within_melee_range: Vec<Entity>,
    enemies_within_missile_range: Vec<Entity>,
    guard_mode_enabled: bool,
    fire_at_will_enabled: bool,
    missile_attack_available: bool,
    can_fire_while_moving: bool,
) -> UnitState {
    // should be current active target
    let TODO = enemies_within_melee_range[0];
    let target_is_dead = false; // TODO

    let engaged_in_melee = !enemies_within_melee_range.is_empty();

    match current_command {
        UnitUserCommand::AttackMelee(cmd_target) => {
            if enemies_within_melee_range.contains(&cmd_target) {
                // priority target should be the user command 
                UnitState::Melee(cmd_target.clone())
            } else if engaged_in_melee {
                UnitState::Melee(TODO)
            } else if target_is_dead {
                UnitState::Idle
            } else {
                // target still alive but out of melee range
                if guard_mode_enabled {
                    UnitState::Moving
                } else {
                    // TODO clear command
                    UnitState::Idle
                }
            }
        }
        UnitUserCommand::AttackMissile(cmd_target) => {
            if engaged_in_melee {
                UnitState::Melee(TODO)
            } else if !missile_attack_available {
                UnitState::Idle
            } else if enemies_within_missile_range.contains(&cmd_target) {
                UnitState::Firing(cmd_target.clone())
            } else {  // target is not within missle range
                if guard_mode_enabled {
                    if let Some(target) = pick_missile_target(
                        &enemies_within_missile_range
                    ) { 
                        if fire_at_will_enabled {
                            // if other targets within missle range, fire at will on
                            // start firing
                            UnitState::Firing(target)
                        } else {
                            UnitState::Idle
                        }
                    } else {
                        UnitState::Idle
                    }
                } else {
                    // no guard mode, chase target
                    if can_fire_while_moving && fire_at_will_enabled {
                        if let Some(target) = pick_missile_target(&enemies_within_missile_range) {
                            UnitState::FiringAndMoving(target)
                        } else {
                            UnitState::Moving
                        }
                    } else {
                        UnitState::Moving
                    }
                 }
            }
        }
        UnitUserCommand::Move(_) => {
            if engaged_in_melee {
                UnitState::Melee(TODO)
            } else if can_fire_while_moving && missile_attack_available {
                if let Some(target) = pick_missile_target(&enemies_within_missile_range) {
                    UnitState::FiringAndMoving(target)
                } else {
                    UnitState::Moving
                }
            } else {
                UnitState::Moving
            }
        }
        UnitUserCommand::None_ => {
            if engaged_in_melee {
                UnitState::Melee(TODO)
            } else {
                if missile_attack_available && fire_at_will_enabled {
                    if let Some(target) = pick_missile_target(&enemies_within_missile_range) {
                        UnitState::Firing(target)    
                    } else {
                        UnitState::Idle
                    }
                } else {
                    UnitState::Idle
                }
            }
        }
    }
}

/// Updates each units state machine
pub fn unit_state_machine_system(
    game_speed: Res<GameSpeed>,
    mut units: Query<&mut UnitComponent>
) {
    if game_speed.is_paused() { return; }

    for mut unit in &mut units.iter() {
        let melee_range_enemies = vec![];
        let missile_range_enemies = vec![];

        unit.state = calculate_next_unit_state(
            &unit.current_command,
            melee_range_enemies,
            missile_range_enemies,
            unit.guard_mode_enabled, 
            unit.guard_mode_enabled,
            unit.is_missile_attack_available(),
            unit.can_fire_while_moving(),
        );
    }
}

/// for each unit, calculates the position of its waypoint
pub fn unit_waypoint_system(
    game_speed: Res<GameSpeed>,
    mut unit_query: Query<(&UnitComponent, &mut WaypointComponent)>,
    target_query: Query<&Transform>,
) {
    if game_speed.is_paused() { return; }

    for (unit, mut waypoint) in &mut unit_query.iter() {
        match &unit.current_command {
            UnitUserCommand::AttackMelee(target) | UnitUserCommand::AttackMissile(target) => {
                let target_translation = target_query
                    .get::<Transform>(target.clone())
                    .expect("Target translation")
                    .translation();
                *waypoint = WaypointComponent::Position(
                    (target_translation.x(), target_translation.y()).into(),
                );
            }
            UnitUserCommand::Move(wp) => {
                // TODO this is unnessecary, but maybe its where its where we put in some pathfinding to determine the next step?
                *waypoint = WaypointComponent::Position(wp.clone());
            }
            UnitUserCommand::None_ => {}
        }
    }
}

// TODO have a separate component for waypoint position for all command types
// that is updated in a separate system, so its calculated separately from the unit movement system
// so we don't run into unique borrow issues
pub fn unit_movement_system(
    game_speed: Res<GameSpeed>,
    time: Res<Time>,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    mut unit_query: Query<(
        Entity,
        &mut UnitComponent,
        &mut Transform,
        &mut RigidBodyHandleComponent,
        &mut ColliderHandleComponent,
        &WaypointComponent,
    )>,
) {
    if game_speed.is_paused() { return; }

    for (entity, unit, mut transform, body_handle, collider_handle, waypoint) in
        &mut unit_query.iter()
    {
        let translation = transform.translation_mut();

        // TODO remove transform here, use rigid body pos
        let unit_pos: XyPos = (translation.x(), translation.y()).into();

        let mut body = bodies.get_mut(body_handle.handle()).expect("body");
        let collider = colliders
            .get_mut(collider_handle.handle())
            .expect("collider");

        // if the unit is going somewhere
        if let UnitState::Moving | UnitState::FiringAndMoving(_) = &unit.state {
            if let Some(dest) = match &unit.current_command {
                UnitUserCommand::AttackMelee(_) | UnitUserCommand::AttackMissile(_) => {
                    if let WaypointComponent::Position(xy) = waypoint {
                        Some(xy)
                    } else {
                        log::error!("attack command without a waypoint!");
                        None
                    }
                }
                UnitUserCommand::Move(_) => {
                    if let WaypointComponent::Position(xy) = waypoint {
                        Some(xy)
                    } else {
                        log::error!("attack command without a waypoint!");
                        None
                    }
                }
                UnitUserCommand::None_ => None,
            } {
                let relative_position = dest.clone() - unit_pos;

                let unit_distance = unit.current_speed() * time.delta_seconds;

                // using length_squared() for totally premature optimization
                let rel_distance_sq = relative_position.length_squared();

                // if we need to keep moving
                if unit_distance.powi(2) < rel_distance_sq {
                    // get direction
                    let direction = relative_position.normalize();

                    // move body
                    let pos = Isometry::translation(
                        body.position.translation.vector.x + (direction.x() * unit_distance),
                        body.position.translation.vector.y + (direction.y() * unit_distance),
                    );

                    body.set_position(pos);
                    collider.set_position_debug(pos);
                } else {
                    // can reach destination, set position to waypoint, transition to idle
                    let pos = Isometry::translation(dest.x(), dest.y());
                    body.set_position(pos);
                    collider.set_position_debug(pos);
                    unit_events.send(UnitInteractionEvent::Proximity(
                        ContactType::UnitWaypointReached(entity),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_unit_state_machine() {
        
    }
}