//! contains setup code
#![allow(dead_code)]

use std::collections::HashMap;


use bevy::{prelude::*, render::pass::ClearColor};


use bevy_rapier2d::physics::{
    ColliderHandleComponent, RapierPhysicsPlugin, RigidBodyHandleComponent,
};
use bevy_rapier2d::rapier::dynamics::{RigidBodyBuilder};
use bevy_rapier2d::rapier::geometry::{ColliderBuilder};

use bevy_rapier2d::render::RapierRenderPlugin;

use itertools::Itertools;

use tntw::combat::*;
use tntw::physics::*;
use tntw::teams::*;
use tntw::ui;
use tntw::units::*;
use tntw::user_input;
use tntw::*;

fn main() {
    env_logger::init();
    App::build()
        .add_default_plugins()
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin) // for debugging
        .add_plugin(tntw::game_speed::GameSpeedPlugin) // for debugging
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_resource(BodyHandleToEntity(HashMap::new()))
        .add_resource(EntityToBodyHandle(HashMap::new()))
        .add_resource(EntityToColliderType(HashMap::new()))
        .add_resource(DebugTimer(Timer::from_seconds(1.0, true)))
        .init_resource::<user_input::InputState>()
        .init_resource::<ui::SelectionMaterials>()
        .init_resource::<ui::HeathBarMaterials>()
        .init_resource::<TeamRelationshipLookup>()
        .add_event::<UnitInteractionEvent>()

        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(user_input::cursor_system.system())
        .add_system(user_input::input_system.system())

        .add_system(unit_event_system.system())
        .add_system(unit_state_machine_system.system())
        .add_system(unit_waypoint_system.system())
        .add_system(unit_movement_system.system())
        .add_system(body_to_entity_system.system())
        .add_system(remove_rigid_body_system.system())
        .add_system(physics_debug_system.system())
        
        .add_system(unit_melee_system.system())
        .add_system(unit_missile_system.system())
        
        .add_system(ui::unit_display_system.system())
        .add_system_to_stage(
            stage::POST_UPDATE,
            unit_proximity_interaction_system.system(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut team_lookup: ResMut<TeamRelationshipLookup>,
    selection_materials: Res<ui::SelectionMaterials>,
    healthbar_materials: Res<ui::HeathBarMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ui::UiStateMaterials {
        idle: materials.add(
            asset_server
                .load("assets/textures/idle.png")
                .unwrap()
                .into(),
        ),
        moving: materials.add(
            asset_server
                .load("assets/textures/move.png")
                .unwrap()
                .into(),
        ),
        moving_fast: materials.add(
            asset_server
                .load("assets/textures/move_fast.png")
                .unwrap()
                .into(),
        ),
        melee: materials.add(
            asset_server
                .load("assets/textures/swords.png")
                .unwrap()
                .into(),
        ),
        firing: materials.add(asset_server.load("assets/textures/bow.png").unwrap().into()), // UPDATED
    });

    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    let unit_start_positions = vec![
        (UnitType::MissileInfantry, 1, 150.0, 0.0),
        (UnitType::MeleeInfantry, 2, -150.0, 0.0),
    ];

    let unit_size = 30.0;
    let state_icon_size = 12.0;

    let mut all_teams = vec![];

    for (ut, team, x, y) in unit_start_positions.into_iter() {
        let unit = UnitComponent::default_from_type(ut, team);

        let body = RigidBodyBuilder::new_dynamic()
            .translation(x, y)
            .can_sleep(false); // things start annoyingly asleep

        // TODO add more colliders when bevy_rapier supports it.
        // for now, missile units cant engage in melee
        let collider = if let AttackType::Melee = &unit.primary_attack_type() {
            ColliderBuilder::cuboid(unit_size / 2.0, unit_size / 2.0).sensor(true)
        } else {
            if let MissileWeapon::Primary(stats) = &unit.missile_weapon {
                ColliderBuilder::ball(stats.range).sensor(true)
            } else {
                unimplemented!();
            }
        };

        all_teams.push(team);

        commands
            .spawn(SpriteComponents {
                material: selection_materials.normal.into(),
                transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                sprite: Sprite::new(Vec2::new(unit_size, unit_size)),
                ..Default::default()
            })
            .with(unit)
            .with(WaypointComponent::default())
            .with(HealthComponent::default())
            .with(NearbyUnitsComponent::default())
            .with_bundle((body, collider))
            // ui state icon
            .with_children(|parent| {
                parent.spawn(SpriteComponents {
                    material: selection_materials.normal.into(),
                    transform: Transform::from_translation(Vec3::new(
                        (unit_size / 2.0) + (state_icon_size / 2.0) + 5.0,
                        (unit_size / 2.0) - (state_icon_size / 2.0),
                        0.0,
                    ))
                    .with_scale(ui::ICON_SCALE),
                    sprite: Sprite::new(Vec2::new(state_icon_size, state_icon_size)),
                    ..Default::default()
                });
            })
            // healthbar
            .with_children(|parent| {
                let xpos = 0.0;
                let ypos = -(unit_size / 2.0) - 5.0;

                // background
                parent.spawn(SpriteComponents {
                    material: healthbar_materials.background.into(),
                    transform: Transform::from_translation(Vec3::new(xpos, ypos, 1.0)),
                    sprite: Sprite::new(Vec2::new(unit_size, 5.0)),
                    ..Default::default()
                });
                // foreground
                parent.spawn(SpriteComponents {
                    material: healthbar_materials.high.into(),
                    transform: Transform::from_translation(Vec3::new(xpos, ypos, 2.0)),
                    sprite: Sprite::new(Vec2::new(unit_size, 5.0)),
                    ..Default::default()
                });
            });
    }

    // populate teams
    for team_pair in all_teams
        .into_iter()
        .tuple_combinations::<(TeamId, TeamId)>()
    {
        if team_pair.0 == team_pair.1 {
            team_lookup.0.insert(team_pair, TeamRelation::Same);
        } else {
            // TODO support allies
            team_lookup.0.insert(team_pair, TeamRelation::Enemy);
        }
    }

    // set up cursor tracker
    let camera = Camera2dComponents::default();
    let e = commands
        .spawn(camera)
        .current_entity()
        .expect("Camera entity");
    commands.insert_resource(user_input::CursorState {
        cursor: Default::default(),
        camera_e: e,
        last_pos: XyPos::default(),
    });
}
