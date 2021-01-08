//! systems and types for proximity interations for units

use bevy::prelude::*;
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent, rapier::dynamics::JointSet,
    rapier::dynamics::RigidBodyHandle, rapier::dynamics::RigidBodySet,
    rapier::geometry::BroadPhase, rapier::geometry::ColliderSet, rapier::geometry::NarrowPhase,
    rapier::pipeline::PhysicsPipeline,
};

use bevy_rapier2d::physics::EventQueue;
use bevy_rapier2d::rapier::geometry::Proximity;

use std::collections::HashMap;

pub enum ColliderType {
    Melee,
    FiringRange,
}

pub struct BodyHandleToEntity(pub HashMap<RigidBodyHandle, Entity>);
pub struct EntityToBodyHandle(pub HashMap<Entity, RigidBodyHandle>);
pub struct EntityToColliderType(pub HashMap<Entity, ColliderType>);

use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum ContactType {
    UnitUnitMeleeEnter(Entity, Entity),
    UnitUnitMeleeExit(Entity, Entity),
    UnitFiringRangeEnter { range_of: Entity, target: Entity },
    UnitFiringRangeExit { range_of: Entity, target: Entity },
}

pub fn unit_proximity_interaction_system(
    bh_to_e: Res<BodyHandleToEntity>,
    e_to_ct: Res<EntityToColliderType>,
    bodies: ResMut<RigidBodySet>,
    events: Res<EventQueue>,
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    units: Query<&UnitComponent>,
) {
    use ColliderType::*;
    // we can can ignore contact events because we are only using sensors, not
    // rigid-body contactors. Sensors only spawn proximity events.
    // while let Ok(contact_event) = events.contact_events.pop() {
    // }

    // contacts stores references by entity rather than RefMut<Unit> to avoid
    // double mutable borrows
    let mut contacts = vec![];

    // prox events are triggered between sensors and colliders (sensor or not)
    while let Ok(prox_event) = events.proximity_events.pop() {
        // new_status is guaranteed to be != prev_status
        match prox_event.new_status {
            Proximity::Disjoint => {
                let b1 = bodies.get(prox_event.collider1).unwrap();
                let b2 = bodies.get(prox_event.collider2).unwrap();
                let e1 = Entity::from_bits(b1.user_data as u64);
                let e2 = Entity::from_bits(b2.user_data as u64);

                if units.get_component::<UnitComponent>(e1).is_ok()
                    && units.get_component::<UnitComponent>(e2).is_ok()
                {
                    match (e_to_ct.0.get(&e1).unwrap(), e_to_ct.0.get(&e2).unwrap()) {
                        (Melee, Melee) => {
                            contacts.push(ContactType::UnitUnitMeleeExit(e1, e2));
                        }
                        (Melee, FiringRange) => {
                            contacts.push(ContactType::UnitFiringRangeExit {
                                range_of: e2,
                                target: e1,
                            });
                        }
                        (FiringRange, Melee) => {
                            contacts.push(ContactType::UnitFiringRangeExit {
                                range_of: e1,
                                target: e2,
                            });
                        }
                        (FiringRange, FiringRange) => (),
                    }
                }
            }
            Proximity::Intersecting => {
                let e1 = *(bh_to_e.0.get(&prox_event.collider1).expect("get"));
                let e2 = *(bh_to_e.0.get(&prox_event.collider2).expect("get"));
                if units.get_component::<UnitComponent>(e1).is_ok()
                    && units.get_component::<UnitComponent>(e2).is_ok()
                {
                    match (e_to_ct.0.get(&e1).unwrap(), e_to_ct.0.get(&e2).unwrap()) {
                        (Melee, Melee) => {
                            contacts.push(ContactType::UnitUnitMeleeEnter(e1, e2));
                        }
                        (Melee, FiringRange) => {
                            contacts.push(ContactType::UnitFiringRangeEnter {
                                range_of: e2,
                                target: e1,
                            });
                        }
                        (FiringRange, Melee) => {
                            contacts.push(ContactType::UnitFiringRangeEnter {
                                range_of: e1,
                                target: e2,
                            });
                        }
                        (FiringRange, FiringRange) => (),
                    }
                }
            }
            // ignore WithinMargin because we don't need any special behaviour for that case.
            Proximity::WithinMargin => (),
        }
    }

    for contact in contacts {
        unit_events.send(UnitInteractionEvent::Proximity(contact));
    }
}

/// Detects when a RigidBodyHandle is removed from an entity, as it despawns
/// And inform rapier about the removal
pub fn remove_rigid_body_system(
    _pipeline: ResMut<PhysicsPipeline>,
    _broad_phase: ResMut<BroadPhase>,
    _narrow_phase: ResMut<NarrowPhase>,
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
        bodies.remove(handle.clone(), &mut colliders, &mut joints);
        bh_to_e.0.remove(handle);
        e_to_bh.0.remove(entity);
    }
}

pub fn physics_debug_system(
    time: Res<Time>,
    mut debug_timer: ResMut<DebugTimer>,
    mut bodies: ResMut<RigidBodySet>,
    colliders: ResMut<ColliderSet>,
    query: Query<(Entity, &RigidBodyHandleComponent)>,
) {
    debug_timer.0.tick(time.delta_seconds());
    if debug_timer.0.finished() {
        for (entity, body_handle) in query.iter() {
            let body = bodies.get_mut(body_handle.handle()).expect("body");
            log::trace!(
                "entity {:?} at ({}, {}). sleeping: {}",
                entity,
                body.position().translation.x,
                body.position().translation.y,
                body.is_sleeping()
            );
        }
        log::trace!("#colliders: {}", colliders.len());
        log::trace!("#bodies: {}", bodies.len());
        for (_idx, _collider) in colliders.iter() {
            // log::trace!(
            //     "collider {:?} at ({}, {})",
            //     idx,
            //     collider.position().translation.x,
            //     collider.position().translation.y
            // );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnterOrExit {
    Enter,
    Exit,
}

impl ContactType {
    pub fn enter_or_exit(&self) -> EnterOrExit {
        match self {
            ContactType::UnitFiringRangeEnter { .. } | ContactType::UnitUnitMeleeEnter(..) => {
                EnterOrExit::Enter
            }
            ContactType::UnitFiringRangeExit { .. } | ContactType::UnitUnitMeleeExit(..) => {
                EnterOrExit::Exit
            }
        }
    }
}
