


use bevy::{prelude::*};


use bevy_rapier2d::physics::{
    ColliderHandleComponent, RigidBodyHandleComponent,
};
use bevy_rapier2d::rapier::dynamics::{RigidBodySet};
use bevy_rapier2d::rapier::geometry::{ColliderSet};
use bevy_rapier2d::rapier::math::Isometry;


use crate::physics::*;

use crate::*;

/// stores units that are within attack (melee or missle) range of the
/// associated unit, used to determining unit behaviour.
#[derive(Debug, Default)]
pub struct NearbyUnitsComponent {
    melee_range: Vec<Entity>,
    missle_range: Vec<Entity>,
}


/// helper function
/// this processes interactions one unit at a time within its own scope
/// so we don't double-borrow the Unit Component
fn process_unit_proximity(
    unit_id: Entity,
    target_id: Entity,
    mut nearbys: &mut Query<&mut NearbyUnitsComponent>,
    e_or_e: EnterOrExit,
    range_type: AttackType,
) {
    let mut nbs = nearbys.get_component_mut::<NearbyUnitsComponent>(unit_id).unwrap();
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

/// helper function
fn process_unit_command(
    unit_id: Entity,
    cmd: UnitUiCommand,
    mut units: &mut Query<&mut UnitComponent>,
) {
    use UnitUiCommand::*;
    let mut unit = units.get_component_mut::<UnitComponent>(unit_id).unwrap();
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


/// handles events that changes the commands and state for each unit.
/// processes the following inputs:
/// - unit proximity interactions
/// - TODO user commands
pub fn unit_event_system(
    mut commands: Commands,
    game_speed: Res<GameSpeed>,
    mut state: Local<UnitInteractionState>,
    events: Res<Events<UnitInteractionEvent>>,
    mut units: Query<&mut UnitComponent>,
    mut nearbys: Query<&mut NearbyUnitsComponent>,
) {
    // TODO maybe this should be running?
    if game_speed.is_paused() {
        return;
    }

    let mut dead_units = vec![];

    // process state updates for units that have new events
    for event in state.event_reader.iter(&events) {
        log::debug!("event: {:?}", &event);
        match event.clone() {
            UnitInteractionEvent::Proximity(contact) => {
                match contact {
                    ContactType::UnitFiringRangeEnter { range_of, target } => {
                        process_unit_proximity(
                            range_of,
                            target,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Ranged,
                        );
                    }
                    ContactType::UnitFiringRangeExit { range_of, target } => {
                        process_unit_proximity(
                            range_of,
                            target,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Ranged,
                        );
                    }
                    ContactType::UnitUnitMeleeExit(e1, e2) => {
                        // separate scopes so we don't double-borrow the Unit component
                        process_unit_proximity(
                            e1,
                            e2,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Melee,
                        );
                        process_unit_proximity(
                            e2,
                            e1,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Melee,
                        );
                    }
                    ContactType::UnitUnitMeleeEnter(e1, e2) => {
                        process_unit_proximity(
                            e1,
                            e2,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Melee,
                        );
                        process_unit_proximity(
                            e2,
                            e1,
                            &mut nearbys,
                            contact.enter_or_exit(),
                            AttackType::Melee,
                        );
                    }
                }
            }
            UnitInteractionEvent::Ui(entity, cmd) => {
                process_unit_command(entity, cmd, &mut units);
            }
            UnitInteractionEvent::UnitWaypointReached(e1) => {
                let mut unit = units.get_component_mut::<UnitComponent>(e1).unwrap();
                unit.current_command = UnitUserCommand::None_;
            }
            UnitInteractionEvent::UnitDied(e) => {
                // the e here is the unit that died, but we also need to cancel existing attack commands
                // for this unit. We could maybe do a reverse lookup? but for now just store and iterate
                // over all units to find ones 
                dead_units.push(e);

            }

        }
    }

    for mut unit in units.iter_mut() {
        for dead in dead_units.iter() {
            
            // clear actively fighting target, if there was any
            if let Some(target) = &unit.state.current_actively_fighting() {
                if dead == target {
                    unit.state.clear_target();
                }
            }
            
            // clear current command if it was on the dead unit
            if let UnitUserCommand::AttackMelee(target) | UnitUserCommand::AttackMissile(target) = &unit.current_command {
                if dead == target {
                    unit.current_command = UnitUserCommand::None_;

                }
            }
        }
    }

    for mut nearby in nearbys.iter_mut() {
        for dead in dead_units.iter() {
            nearby.melee_range.retain(|e| e != dead);
            nearby.missle_range.retain(|e| e != dead);
        }
    }

    for dead in dead_units.iter() {
        commands.despawn_recursive(dead.clone());
    }
}

/// Returns None if no targets available
/// TODO make this fancier
pub fn pick_missile_target(available_targets: &Vec<Entity>) -> Option<Entity> {
    available_targets.get(0).map(|e| e.clone())
}

/// Returns None if no targets available
/// TODO make this fancier
pub fn pick_melee_target(available_targets: &Vec<Entity>) -> Option<Entity> {
    available_targets.get(0).map(|e| e.clone())
}

pub fn calculate_next_unit_state_and_target(
    current_command: &UnitUserCommand,
    enemies_within_melee_range: &Vec<Entity>,
    enemies_within_missile_range: &Vec<Entity>,
    guard_mode_enabled: bool,
    fire_at_will_enabled: bool,
    missile_attack_available: bool,
    can_fire_while_moving: bool,
    currently_fighting: Option<Entity>
) -> UnitState {
    let next_melee_target = || {
        pick_melee_target(enemies_within_melee_range)
    };

    let next_missile_target = || {
        pick_missile_target(enemies_within_missile_range)
    };

    match current_command {
        UnitUserCommand::AttackMelee(cmd_target) => {
            if enemies_within_melee_range.contains(&cmd_target) {
                // priority target should be the user command
                UnitState::Melee(Some(cmd_target.clone()))
            } else if currently_fighting.is_some() {
                // keep fighting the same person as last round
                UnitState::Melee(currently_fighting)
            } else if let Some(next_target) = next_melee_target() {
                // pick someone else
                UnitState::Melee(Some(next_target))
            } else {
                // no one else nearby, target still alive outside of melee range
                if guard_mode_enabled {
                    UnitState::Moving
                } else {
                    // TODO clear command
                    UnitState::Idle
                }
            }
        }
        UnitUserCommand::AttackMissile(cmd_target) => {
            if let Some(next_melee) = next_melee_target() {
                UnitState::Melee(Some(next_melee))
            } else if !missile_attack_available {
                // out of ammo
                UnitState::Idle
            } else if enemies_within_missile_range.contains(&cmd_target) {
                // prioritize user-given target
                UnitState::Firing(Some(cmd_target.clone()))
            } else {  
                // target is not within missle range
                if guard_mode_enabled {  
                    // guard mode, don't move
                    let next_target = next_missile_target();
                    if fire_at_will_enabled && next_target.is_some() {
                        // if other targets within missle range & fire at will on
                        UnitState::Firing(next_target)
                    } else {
                        UnitState::Idle
                    }
                } else {
                    // no guard mode, chase target
                    let next_target = next_missile_target();
                    if can_fire_while_moving && fire_at_will_enabled && next_target.is_some() {
                        // special case for horse archers
                        UnitState::FiringAndMoving(next_target)
                    } else {
                        UnitState::Moving
                    }
                }
            }
        }
        UnitUserCommand::Move(_) => {
            let melee_target = next_melee_target();
            let missile_target = next_missile_target();
            if melee_target.is_some() {
                UnitState::Melee(melee_target)
            } else if can_fire_while_moving && missile_attack_available && missile_target.is_some() {
                UnitState::FiringAndMoving(missile_target)
            } else {
                UnitState::Moving
            }
        }
        UnitUserCommand::None_ => {
            let next_melee = next_melee_target();
            if next_melee.is_some() {
                UnitState::Melee(next_melee)
            } else {
                let next_missile = next_missile_target();
                if missile_attack_available && fire_at_will_enabled && next_missile.is_some() {
                    UnitState::Firing(next_missile)
                } else {
                    UnitState::Idle
                }
            }
        }
    }
}

/// Updates each units state machine
pub fn unit_state_machine_system(game_speed: Res<GameSpeed>, mut units: Query<(&mut UnitComponent, &NearbyUnitsComponent, &MissileWeaponComponent)>) {
    if game_speed.is_paused() {
        return;
    }

    for (mut unit, nearbys, missile) in units.iter_mut() {

        let new_state = calculate_next_unit_state_and_target(
            &unit.current_command,
            &nearbys.melee_range,
            &nearbys.missle_range,
            unit.guard_mode_enabled,
            unit.fire_at_will,
            missile.is_missile_attack_available(),
            unit.can_fire_while_moving(),
            unit.state.current_actively_fighting(),
        );

        if unit.state != new_state {
            log::debug!("Unit state transition {:?}->{:?} with command {:?}", unit.state, new_state, unit.current_command);
        }

        unit.state = new_state;
    }
}

/// for each unit, calculates the position of its waypoint
pub fn unit_waypoint_system(
    game_speed: Res<GameSpeed>,
    mut unit_query: Query<(&UnitComponent, &mut WaypointComponent)>,
    target_query: Query<&Transform>,
) {
    if game_speed.is_paused() {
        return;
    }

    for (unit, mut waypoint) in unit_query.iter_mut() {
        match &unit.current_command {
            UnitUserCommand::AttackMelee(target) | UnitUserCommand::AttackMissile(target) => {
                let target_translation = target_query
                    .get_component::<Transform>(target.clone())
                    .expect("Target translation")
                    .translation;
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
    if game_speed.is_paused() {
        return;
    }

    for (entity, unit, mut transform, body_handle, collider_handle, waypoint) in
        unit_query.iter_mut()
    {
        let translation = transform.translation;

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
                    unit_events.send(UnitInteractionEvent::UnitWaypointReached(
                        entity
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_unit_state_machine() {}
}
