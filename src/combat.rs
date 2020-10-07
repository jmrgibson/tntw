//! Combat must happen AFTER states have been calcualted so that we aren't in combat with a
//! unit that got cleaned up at the end of the last loop

// TODO unit timers on how fast combat should happen

use bevy::prelude::*;

use crate::*;

const ATTACK_DAMAGE: f32 = 1.0;
const MISSILE_DAMAGE: f32 = 3.3;

pub fn unit_melee_system(
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    game_speed: Res<GameSpeed>,
    mut unit_query: Query<&UnitComponent>,
    health_query: Query<(&mut HealthComponent)>,
) {
    if game_speed.is_paused() {
        return;
    }

    for unit in &mut unit_query.iter() {
        if let UnitState::Melee(Some(target)) = unit.state {
            let mut target_heath = health_query.get_mut::<HealthComponent>(target).unwrap();
            target_heath.current_health -= ATTACK_DAMAGE;

            if target_heath.current_health < 0.0 {
                log::info!("unit dead!");
                unit_events.send(UnitInteractionEvent::UnitDied(target));
            }
        }
    }
}

pub fn unit_missile_system(
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    game_speed: Res<GameSpeed>,
    mut unit_query: Query<&UnitComponent>,
    health_query: Query<(&mut HealthComponent)>,
) {
    if game_speed.is_paused() {
        return;
    }
    
    for unit in &mut unit_query.iter() {
        if let UnitState::Firing(Some(target)) | UnitState::FiringAndMoving(Some(target)) = unit.state {
            let mut target_heath = health_query.get_mut::<HealthComponent>(target).unwrap();
            target_heath.current_health -= MISSILE_DAMAGE;

            if target_heath.current_health < 0.0 {
                log::info!("unit dead!");
                unit_events.send(UnitInteractionEvent::UnitDied(target));
            }
        }
    }
}
