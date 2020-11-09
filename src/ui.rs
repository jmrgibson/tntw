//! Systems and structs for the users interface

use bevy::prelude::*;

use crate::{HealthComponent, UnitComponent, UnitUiState};

pub const ICON_SCALE: f32 = 1.2;

pub struct SelectionMaterials {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
    pub selected: Handle<ColorMaterial>,
}

pub struct HeathBarMaterials {
    pub high: Handle<ColorMaterial>,
    pub medium: Handle<ColorMaterial>,
    pub low: Handle<ColorMaterial>,
    pub background: Handle<ColorMaterial>,
}

pub struct UiStateMaterials {
    pub idle: Handle<ColorMaterial>,
    pub moving: Handle<ColorMaterial>,
    pub moving_fast: Handle<ColorMaterial>,
    pub melee: Handle<ColorMaterial>,
    pub firing: Handle<ColorMaterial>,
}

pub fn unit_display_system(
    selection_materials: Res<SelectionMaterials>,
    healthbar_materials: Res<HeathBarMaterials>,
    icon_materials: Res<UiStateMaterials>,
    mut unit_query: QuerySet<(
        Query<(
            &UnitComponent,
            // &HealthComponent,
            // &Handle<ColorMaterial>,
            &Children,
        )>,
        Query<&mut Handle<ColorMaterial>>,
        Query<(
            &HealthComponent,
            &Children
        )>,
    )>,
    // mut icon_query: Query<&mut Handle<ColorMaterial>>,
    mut sprite_query: Query<&mut Sprite>,
    mut transform_query: Query<&mut Transform>,
) {
    // for (unit, children) in unit_query.q0().iter() {
    //     // state icon
    //     {
    //         let mut state_icon = unit_query
    //             .q1_mut()
    //             .get_component_mut::<Handle<ColorMaterial>>(children[0])
    //             .unwrap();
    //         *state_icon = match unit.ui_state() {
    //             UnitUiState::MovingSlow => icon_materials.moving.clone(),
    //             UnitUiState::MovingFast => icon_materials.moving_fast.clone(),
    //             UnitUiState::Melee => icon_materials.melee.clone(),
    //             UnitUiState::Firing => icon_materials.firing.clone(),
    //             _ => icon_materials.idle.clone(),
    //         };
    //     }
    // }

    //     // selection status
    //     {
    //         *material = if unit.is_selected() {
    //             selection_materials.selected.clone()
    //         } else {
    //             selection_materials.normal.clone()
    //         };
    //     }

    for (health, children) in unit_query.q2_mut().iter_mut() {
        // healthbar
        {
            // shrink healthbar, first get background as reference
            let max_width = { sprite_query.get_component::<Sprite>(children[1]).unwrap().size.x() };
            let left_anchor = {
                transform_query
                    .get_component::<Transform>(children[1])
                    .unwrap()
                    .translation
                    .x()
                    - (max_width / 2.0)
            };

            // then update actual healtbar
            let mut foreground = sprite_query.get_component_mut::<Sprite>(children[2]).unwrap();

            let bar_size = max_width * health.ratio();
            foreground.size.set_x(bar_size);

            transform_query
                .get_component_mut::<Transform>(children[2])
                .unwrap()
                .translation
                .set_x(left_anchor + bar_size / 2.0);

            // update color
            let mut healthbar = unit_query
                .q1_mut()
                .get_component_mut::<Handle<ColorMaterial>>(children[2])
                .unwrap();
            *healthbar = healthbar_materials.from_ratio(health.ratio());
        }
    }
}

impl FromResources for SelectionMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources
            .get_mut::<Assets<ColorMaterial>>()
            .expect("Colour resource");
        SelectionMaterials {
            normal: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
            hovered: materials.add(Color::rgb(0.05, 0.05, 0.05).into()),
            selected: materials.add(Color::rgb(0.8, 0.8, 0.1).into()),
        }
    }
}

impl FromResources for HeathBarMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources
            .get_mut::<Assets<ColorMaterial>>()
            .expect("Colour resource");
        HeathBarMaterials {
            high: materials.add(Color::rgb(0.1, 0.9, 0.1).into()),
            medium: materials.add(Color::rgb(0.9, 0.9, 0.1).into()),
            low: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            background: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
        }
    }
}

impl HeathBarMaterials {
    pub fn from_ratio(&self, health_ratio: f32) -> Handle<ColorMaterial> {
        if health_ratio >= 0.75 {
            self.high.clone()
        } else if health_ratio >= 0.25 {
            self.medium.clone()
        } else {
            self.low.clone()
        }
    }
}

impl HealthComponent {
    pub fn ratio(&self) -> f32 {
        self.current_health / self.max_health
    }
}
