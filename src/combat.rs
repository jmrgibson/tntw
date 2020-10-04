use bevy::{
    prelude::*,
};

use crate::*;

const ATTACK_DAMAGE: f32 = 1.0;

pub fn unit_melee_system(
    mut commands: Commands,
    mut unit_query: Query<&mut UnitComponent>,
    health_query: Query<(Entity, &mut HealthComponent)>
) {
    for unit in &mut unit_query.iter() {
        if let UnitCurrentAction::Melee(target) = unit.current_action {
            let mut target_heath = health_query.get_mut::<HealthComponent>(target).unwrap();
            target_heath.current_health -= ATTACK_DAMAGE;
            
            if target_heath.current_health < 0.0 {
                log::info!("unit dead!");
                commands.despawn(target);
            }
        }
    }
} 