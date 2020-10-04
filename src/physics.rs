use bevy::prelude::*;
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent, rapier::dynamics::JointSet,
    rapier::dynamics::RigidBodyHandle, rapier::dynamics::RigidBodySet,
    rapier::geometry::BroadPhase, rapier::geometry::ColliderSet, rapier::geometry::NarrowPhase,
    rapier::pipeline::PhysicsPipeline,
};

use bevy_rapier2d::physics::{EventQueue};
use bevy_rapier2d::rapier::dynamics::{RigidBodyBuilder};
use bevy_rapier2d::rapier::geometry::{Proximity, ColliderBuilder};

use std::collections::HashMap;
pub struct BodyHandleToEntity(pub HashMap<RigidBodyHandle, Entity>);
pub struct EntityToBodyHandle(pub HashMap<Entity, RigidBodyHandle>);

use crate::*;

#[derive(Debug,Copy, Clone)]
pub enum ContactType {
    UnitUnitMeleeEngage(Entity, Entity),
    UnitUnitMeleeDisengage(Entity, Entity),
    UnitWaypointReached(Entity),
}


pub fn unit_proximity_interaction_system(
    bh_to_e: Res<BodyHandleToEntity>,
    events: Res<EventQueue>,
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    units: Query<&UnitComponent>, 
) {
    // we can can ignore contact events because we are only using sensors, not
    // rigid contactors
    // while let Ok(contact_event) = events.contact_events.pop() {
    // }
    
    // contacts stores references by entity rather than RefMut<Unit> to avoid
    // double mutable borrows
    let mut contacts = vec![];

    // prox events are triggered between sensors and colliders (sensor or not)
    while let Ok(prox_event) = events.proximity_events.pop() {  
        // we can ignore WithinMargin because we don't need any special behaviour for that case.
        // new_status is guaranteed to be != prev_status
        match prox_event.new_status {
            Proximity::Disjoint => {
                let e1 = *(bh_to_e.0.get(&prox_event.collider1).expect("get"));
                let e2 = *(bh_to_e.0.get(&prox_event.collider2).expect("get"));
                if units.get::<UnitComponent>(e1).is_ok() && units.get::<UnitComponent>(e2).is_ok() {
                    contacts.push(ContactType::UnitUnitMeleeDisengage(e1, e2));
                }
            },
            Proximity::Intersecting => {
                let e1 = *(bh_to_e.0.get(&prox_event.collider1).expect("get"));
                let e2 = *(bh_to_e.0.get(&prox_event.collider2).expect("get"));
                if units.get::<UnitComponent>(e1).is_ok() && units.get::<UnitComponent>(e2).is_ok() {
                    contacts.push(ContactType::UnitUnitMeleeEngage(e1, e2));
                }
            },
            Proximity::WithinMargin => (),
        } 
    }

    for contact in contacts {
        unit_events.send(UnitInteractionEvent::Proximity(contact));
    }
}

/// Keeps BodyHandleToEntity resource in sync.
// TODO: handle removals.
pub fn body_to_entity_system(
    mut bh_to_e: ResMut<BodyHandleToEntity>,
    mut e_to_bh: ResMut<EntityToBodyHandle>,
    mut added: Query<(Entity, Added<RigidBodyHandleComponent>)>,
) {
    for (entity, body_handle) in &mut added.iter() {
        log::debug!("new rigid body");
        bh_to_e.0.insert(body_handle.handle(), entity);
        e_to_bh.0.insert(entity, body_handle.handle());
    }
}


/// Detects when a RigidBodyHandle is removed from an entity, as it despawns
/// And inform rapier about the removal
pub fn remove_rigid_body_system(
    mut pipeline: ResMut<PhysicsPipeline>,
    mut broad_phase: ResMut<BroadPhase>,
    mut narrow_phase: ResMut<NarrowPhase>,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    mut joints: ResMut<JointSet>,
    mut e_to_bh: ResMut<EntityToBodyHandle>,
    mut bh_to_e: ResMut<BodyHandleToEntity>,
    query: Query<&RigidBodyHandleComponent>,
) {
    for entity in query.removed::<RigidBodyHandleComponent>().iter() {
        log::debug!("removed rigid body");
        let handle = e_to_bh.0.get(entity).unwrap();
        pipeline.remove_rigid_body(
            *handle,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut joints,
        );
        bh_to_e.0.remove(handle);
        e_to_bh.0.remove(entity);
    }
}

pub fn physics_debug_system(
    time: Res<Time>,
    mut debug_timer: ResMut<DebugTimer>,
    mut bodies: ResMut<RigidBodySet>,
    colliders: ResMut<ColliderSet>,
    mut query: Query<(Entity, &RigidBodyHandleComponent)>,
) {
    debug_timer.0.tick(time.delta_seconds);
    if debug_timer.0.finished {
        for (entity, body_handle) in &mut query.iter() {
            let body = bodies.get_mut(body_handle.handle()).expect("body");
            log::trace!("entity {:?} at ({}, {}). sleeping: {}", entity, body.position.translation.x, body.position.translation.y, body.is_sleeping());
        }
        log::trace!("#colliders: {}", colliders.len());
        log::trace!("#bodies: {}", bodies.len());
        for (idx, collider) in colliders.iter() {
            log::trace!("collider {:?} at ({}, {})", idx, collider.position().translation.x, collider.position().translation.y);
        }
    }
}