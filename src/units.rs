
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

/// handles events that changes the commands and state for each unit.
/// processes the following inputs:
/// - unit proximity interactions
/// - TODO user commands
pub fn unit_event_system(
    mut state: Local<UnitInteractionState>,
    events: Res<Events<UnitInteractionEvent>>,
    units: Query<&mut UnitComponent>,
) {
    /// this processes interactions one unit at a time within its own scope
    /// so we don't double-borrow the Unit Component
    fn process_unit_disengage(
        unit_id: Entity,
        target_id: Entity,
        units: &Query<&mut UnitComponent>,
    ) {
        let mut unit = units.get_mut::<UnitComponent>(unit_id).unwrap();
        if unit.guard_mode_enabled {
            log::debug!("disengaging in melee");
            // state goes to idle, user command gets cleared
            unit.current_action = UnitCurrentAction::Idle;
            unit.current_command = UnitUserCommand::None_;
        } else {
            // state goes to moving, command to chase, speed to fast.
            log::debug!("pursuing unit");
            unit.current_action = UnitCurrentAction::Moving;
            unit.current_command = UnitUserCommand::Attack(target_id);
            unit.is_running = true;
        }
    }

    fn process_unit_engage(unit_id: Entity, target_id: Entity, units: &Query<&mut UnitComponent>) {
        log::debug!(
            "engaging in melee between {:?} and {:?}",
            unit_id,
            target_id
        );
        let mut unit = units.get_mut::<UnitComponent>(unit_id).unwrap();
        unit.current_action = UnitCurrentAction::Melee(target_id);
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
                unit.current_command = UnitUserCommand::Attack(target);
                unit.is_running = speed == UnitUiSpeedCommand::Run;
            }
            Move(pos, speed) => {
                unit.current_command = UnitUserCommand::Move(pos);
                unit.current_action = UnitCurrentAction::Moving;
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
                    ContactType::UnitUnitMeleeDisengage(e1, e2) => {
                        // separate scopes so we don't double-borrow the Unit component
                        process_unit_disengage(e1, e2, &units);
                        process_unit_disengage(e2, e1, &units);
                    }
                    ContactType::UnitUnitMeleeEngage(e1, e2) => {
                        process_unit_engage(e1, e2, &units);
                        process_unit_engage(e2, e1, &units);
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
/// for each unit, calculates the position of its waypoint
pub fn unit_waypoint_system(
    mut unit_query: Query<(&UnitComponent, &mut WaypointComponent)>,
    target_query: Query<&Transform>,
) {
    for (unit, mut waypoint) in &mut unit_query.iter() {
        match &unit.current_command {
            UnitUserCommand::Attack(target) => {
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
        if let UnitCurrentAction::Moving = &unit.current_action {
            if let Some(dest) = match &unit.current_command {
                UnitUserCommand::Attack(_) => {
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
