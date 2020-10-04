//! Systems and structs for the users interface

use bevy::{prelude::*, render::pass::ClearColor};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

use crate::{UnitComponent, UnitUiState, HealthComponent};

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
    icon_materials: Res<UiStateMaterials>,
    mut unit_query: Query<(&UnitComponent, &HealthComponent, &mut Handle<ColorMaterial>, &Children)>,
    icon_query: Query<&mut Handle<ColorMaterial>>,
) {
    for (unit, health, mut material, children) in &mut unit_query.iter() {
        let mut state_icon = icon_query.get_mut::<Handle<ColorMaterial>>(children[0]).unwrap();
        *state_icon = match unit.ui_state() {
            UnitUiState::MovingSlow => icon_materials.moving,
            UnitUiState::MovingFast => icon_materials.moving_fast,
            UnitUiState::Melee => icon_materials.melee,
            UnitUiState::Firing => icon_materials.firing,
            _ => icon_materials.idle,
        };
        *material = if unit.is_selected() {
            selection_materials.selected
        } else {
            selection_materials.normal
        };
    }
}

impl FromResources for SelectionMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().expect("Colour resource");
        SelectionMaterials {
            normal: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
            hovered: materials.add(Color::rgb(0.05, 0.05, 0.05).into()),
            selected: materials.add(Color::rgb(0.8, 0.8, 0.1).into()),
        }
    }
}

impl FromResources for HeathBarMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().expect("Colour resource");
        HeathBarMaterials {
            high: materials.add(Color::rgb(0.1, 0.9, 0.1).into()),
            medium: materials.add(Color::rgb(0.9, 0.9, 0.1).into()),
            low: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
        }
    }
}

impl HealthComponent {
    pub fn as_color(&self, mats: &Res<HeathBarMaterials>) -> Handle<ColorMaterial> {
        let ratio= self.current_health /  self.max_health;
        if ratio >= 0.75 {
            mats.high
        } else if ratio >= 0.25  {
            mats.medium
        } else {
            mats.low
        }
    }
}