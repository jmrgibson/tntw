#![allow(dead_code)]

use std::time::{Duration, Instant};

use bevy::{prelude::*, render::pass::ClearColor};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;


pub type XyPos = Vec2;

pub struct Unit {
    max_speed: f32,
    is_selected: bool,
}

struct SelectionMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
}

impl FromResources for SelectionMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().expect("Colour resource");
        SelectionMaterials {
            normal: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
            hovered: materials.add(Color::rgb(0.05, 0.05, 0.05).into()),
            selected: materials.add(Color::rgb(0.1, 0.5, 0.1).into()),
        }
    }
}

#[derive(Default)]
struct UiStateMaterials {
    idle: Handle<ColorMaterial>,
    moving: Handle<ColorMaterial>,
    moving_fast: Handle<ColorMaterial>,
}

fn main() {
    env_logger::init();
    App::build()
        .add_default_plugins()
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .init_resource::<SelectionMaterials>()
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(unit_display_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    selection_materials: Res<SelectionMaterials>,
    asset_server: Res<AssetServer>,
) {
    
    commands.insert_resource(UiStateMaterials {
        idle: asset_server.load("assets/textures/idle.png").unwrap(),
        moving: asset_server.load("assets/textures/move.png").unwrap(),
        moving_fast: asset_server.load("assets/textures/move_fast.png").unwrap(),
    });

    let unit_start_positions = vec![(50.0, 0.0), (-50.0, 0.0)];
 
    for (x, y) in unit_start_positions.into_iter() {
        commands
            .spawn(SpriteComponents {
                material: selection_materials.normal.into(),
                transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                sprite: Sprite::new(Vec2::new(30.0, 30.0)),
                ..Default::default()
            })
            .with(Unit::default())
            ;
    }
}

fn unit_display_system(
    selection_materials: Res<SelectionMaterials>,
    icon_materials: Res<UiStateMaterials>,
    mut unit_query: Query<(&Unit, &mut Handle<ColorMaterial>)>,
) {
    for (unit, mut material) in &mut unit_query.iter() {
        
        // fine
        *material = selection_materials.selected;

        // causes crash
        *material = icon_materials.moving_fast;
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            is_selected: false,
            max_speed: 5.0,
        }
    }
}

impl Unit {
    pub fn is_selected(&self) -> bool {
        self.is_selected
    }
}