//! Combat must happen AFTER states have been calcualted so that we aren't in combat with a
//! unit that got cleaned up at the end of the last loop

// TODO unit timers on how fast combat should happen

use bevy::prelude::*;

use crate::*;

pub fn unit_melee_system(
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    game_speed: Res<GameSpeed>,
    mut unit_query: Query<(&UnitComponent, &CombatComponent)>,
    health_query: Query<&mut HealthComponent>,
    target_query: Query<(&CombatComponent)>
) {
    if game_speed.is_paused() {
        return;
    }
    
    for (unit, source) in &mut unit_query.iter() {
        if let UnitState::Melee(Some(target)) = unit.state {
            let mut target_heath = health_query.get_mut::<HealthComponent>(target).unwrap();
            let mut target_combat = target_query.get::<CombatComponent>(target).unwrap();
            target_heath.current_health -= calc_damage(source, &target_combat);
            
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
    mut unit_query: Query<(&UnitComponent, &CombatComponent)>,
    health_query: Query<(&mut HealthComponent)>,
    target_query: Query<(&CombatComponent)>
) {
    if game_speed.is_paused() {
        return;
    }
    
    for (unit, source) in &mut unit_query.iter() {
        if let UnitState::Firing(Some(target)) | UnitState::FiringAndMoving(Some(target)) = unit.state {
            let mut target_heath = health_query.get_mut::<HealthComponent>(target).unwrap();
            let mut target_combat = target_query.get::<CombatComponent>(target).unwrap();
            target_heath.current_health -= calc_damage(source, &target_combat);

            if target_heath.current_health < 0.0 {
                log::info!("unit dead!");
                unit_events.send(UnitInteractionEvent::UnitDied(target));
            }
        }
    }
}

/// TODD make determinisic
/// AP damage is always applied. Armour is rolled between 0-100% of base armour value, 
/// then subtracted from source normal attack damage
fn calc_damage(source: &CombatComponent, target: &CombatComponent) -> f32 {
    source.normal_damage 
        - (target.armour * rand::random::<f32>()) 
        + source.ap_damage
}