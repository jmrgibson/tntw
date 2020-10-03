#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::{prelude::*, render::pass::ClearColor};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;
use bevy_rapier2d::physics::{RapierPhysicsPlugin, RigidBodyHandleComponent, ColliderHandleComponent};
use bevy_rapier2d::rapier::dynamics::{RigidBodyBuilder, RigidBodySet};
use bevy_rapier2d::rapier::geometry::{Proximity, ColliderBuilder, ColliderSet};
use bevy_rapier2d::rapier::math::{Isometry};
use bevy_rapier2d::render::RapierRenderPlugin;


use tntw::{Unit, UnitCommands, UnitCurrentCommand, UnitState, Waypoint, XyPos, DebugTimer};
use tntw::ui;
use tntw::physics::*;

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);
const DRAG_SELECT_MIN_BOX: f32 = 25.0;

fn main() {
    env_logger::init();
    App::build()
        .add_default_plugins()
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin) // for debugging
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_resource(BodyHandleToEntity(HashMap::new()))
        .add_resource(EntityToBodyHandle(HashMap::new()))
        .add_resource(DebugTimer(Timer::from_seconds(1.0, true)))
        .init_resource::<InputState>()
        .init_resource::<ui::SelectionMaterials>()
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(cursor_system.system())
        .add_system(input_system.system())
        .add_system(unit_waypoint_system.system())
        .add_system(unit_movement_system.system())
        // .add_system(unit_proximity_interaction_system.system())
        .add_system(body_to_entity_system.system())
        .add_system(remove_rigid_body_system.system())
        .add_system(physics_debug_system.system())
        .add_system(ui::unit_display_system.system())
        .add_system_to_stage(stage::POST_UPDATE, unit_proximity_interaction_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    selection_materials: Res<ui::SelectionMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {

    commands.insert_resource(ui::UiStateMaterials {
        idle: materials.add(asset_server.load("assets/textures/idle.png").unwrap().into()),  // UPDATED
        moving: materials.add(asset_server.load("assets/textures/move.png").unwrap().into()),  // UPDATED
        moving_fast: materials.add(asset_server.load("assets/textures/move_fast.png").unwrap().into()), // UPDATED
    });

    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    let unit_start_positions = vec![(50.0, 0.0), (-50.0, 0.0)];

    let unit_size = 30.0;
    let state_icon_size = 12.0;
    
    for (x, y) in unit_start_positions.into_iter() {
        
        let body = RigidBodyBuilder::new_dynamic()
            .translation(x, y)
            .can_sleep(false); // things start annoyingly asleep
        let collider = ColliderBuilder
            ::cuboid(unit_size, unit_size)
            .sensor(true);

        commands
            .spawn(SpriteComponents {
                material: selection_materials.normal.into(),
                transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                sprite: Sprite::new(Vec2::new(unit_size, unit_size)),
                ..Default::default()
            })
            .with(Unit::default())
            .with(Waypoint::default())
            .with_bundle((body, collider))
            .with_children(|parent| {
                parent.spawn(SpriteComponents {
                    material: selection_materials.normal.into(),
                    transform: Transform::from_translation(Vec3::new(
                        (unit_size / 2.0) + (state_icon_size / 2.0) + 5.0,
                        (unit_size / 2.0) - (state_icon_size / 2.0),
                        0.0
                    )).with_scale(ui::ICON_SCALE),
                    sprite: Sprite::new(Vec2::new(state_icon_size, state_icon_size)),
                    ..Default::default()
                });
            })
            ;
    }

    // set up cursor tracker
    let camera = Camera2dComponents::default();
    let e = commands.spawn(camera).current_entity().expect("Camera entity");
    commands.insert_resource(CursorState {
        cursor: Default::default(),
        camera_e: e,
        last_pos: XyPos::default(),
    });

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);

    commands
        // left
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // right
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y() + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // top
        .spawn(SpriteComponents {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x() + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid);
}

enum Collider {
    Solid,
}

struct InputState {
    keys: EventReader<KeyboardInput>,
    cursor: EventReader<CursorMoved>,
    motion: EventReader<MouseMotion>,
    mousebtn: EventReader<MouseButtonInput>,
    scroll: EventReader<MouseWheel>,
    /// ie, ctrl+select
    is_multi_select_on: bool,
    /// ie, shift+select
    is_toggle_select_on: bool,
    /// for double-click events
    last_mouse_action: Option<(Instant, MouseButton)>,
    /// should this be an option?
    drag_select_start: Option<XyPos>,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            keys: EventReader::default(),
            cursor: EventReader::default(),
            motion: EventReader::default(),
            mousebtn: EventReader::default(),
            scroll: EventReader::default(),
            is_multi_select_on: false,
            is_toggle_select_on: false,
            last_mouse_action: None,
            drag_select_start: None,
        }
    }
}

/// TODO handle rotation, does this also handle dynamic sprite sizing?
/// TODO there is most certainly a better way of doing this math
fn is_position_within_sprite(
    position_to_check: &XyPos,
    sprite_position: &Vec3,
    sprite: &Sprite,
) -> bool {
    position_to_check.x() < (sprite_position.x() + sprite.size.x())
        && position_to_check.x() > (sprite_position.x() - sprite.size.x())
        && position_to_check.y() < (sprite_position.y() + sprite.size.y())
        && position_to_check.y() > (sprite_position.y() - sprite.size.y())
}

/// TODO  also fairly gross
fn is_translation_within_box(
    position_to_check: &Vec3,
    corner: &Vec2,
    end: &Vec2,
) -> bool {
    let in_x = if end.x() - corner.x() > 0.0 {
        corner.x() < position_to_check.x() && position_to_check.x() < end.x()
    } else {
        corner.x() > position_to_check.x() && position_to_check.x() > end.x()
    };
    let in_y = if end.y() - corner.y() > 0.0 {
        corner.y() < position_to_check.y() && position_to_check.y() < end.y()
    } else {
        corner.y() > position_to_check.y() && position_to_check.y() > end.y()
    };
    in_x && in_y
}

pub enum MouseCommand {
    SingleSelect(XyPos),
    DragSelect{start: XyPos, end: XyPos },
    /// attack/move
    Action(XyPos),
}

fn input_system(
    mut state: ResMut<InputState>,
    cursor: Res<CursorState>,
    ev_keys: Res<Events<KeyboardInput>>,
    // ev_cursor: Res<Events<CursorMoved>>,
    // ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    // ev_scroll: Res<Events<MouseWheel>>,
    mut query: Query<(Entity, &mut Unit, &Transform, &Sprite, &mut Waypoint)>,
) {
    let mut new_commands: Vec<UnitCommands> = Vec::new();

    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            // on press
            if let Some(key) = ev.key_code {
                match key {
                    KeyCode::S => new_commands.push(UnitCommands::Stop),
                    KeyCode::R => new_commands.push(UnitCommands::ToggleSpeed),
                    KeyCode::LShift => state.is_toggle_select_on = true,
                    KeyCode::LControl => state.is_multi_select_on = true,
                    _ => (),
                };
            }
        } else {
            // on release
            if let Some(key) = ev.key_code {
                match key {
                    KeyCode::LShift => state.is_toggle_select_on = false,
                    KeyCode::LControl => state.is_multi_select_on = false,
                    _ => (),
                }
            }
        }
    }

    // mouse input
    let mut mouse_command: Option<MouseCommand> = None;
    let mut is_double_click = false;

    for ev in state.mousebtn.iter(&ev_mousebtn) {
        // get last cursor position
        let mouse_position = cursor.last_pos.clone(); 
        // mouse_pos.replace(cursor.last_pos.clone());

        if ev.state.is_pressed() {
            // process on click

            if ev.button == MouseButton::Left {
                // start drag-select action
                state.drag_select_start.replace(mouse_position.clone());
            } else {
                state.drag_select_start = None;
            }
            // TODO maybe have right click drag actions for movement paths?
        } else {
            // process on release

            // double-click logic
            if let Some((prev_time, prev_button)) = state.last_mouse_action {
                if (Instant::now() - prev_time) < DOUBLE_CLICK_WINDOW && ev.button == prev_button {
                    is_double_click = true;
                }
            };

            state
                .last_mouse_action
                .replace((Instant::now(), ev.button.clone()));

            // l/r command logic
            if ev.button == MouseButton::Right {
                mouse_command.replace(MouseCommand::Action(mouse_position));
            } else if ev.button == MouseButton::Left {
                if let Some(start) = state.drag_select_start {
                    log::debug!("drag select");

                    // TODO replace with proper AABB overlap
                    if (start.x() - mouse_position.x()).abs() > DRAG_SELECT_MIN_BOX && (start.y() - mouse_position.y()).abs() > DRAG_SELECT_MIN_BOX {
                        mouse_command.replace(MouseCommand::DragSelect{start, end: mouse_position});
                    } else {
                        log::debug!("single select");
                        mouse_command.replace(MouseCommand::SingleSelect(mouse_position));
                    }
                } else {
                    log::debug!("single select");
                    mouse_command.replace(MouseCommand::SingleSelect(mouse_position));
                }
            } else {
                // don't care about other buttons
                // TODO use an actual command enum
            }
        }
    }

    // determine if any units were part of the selection
    let mut selection_targets: Vec<Entity> = Vec::new();

    // TODO abstract the mouse logic into another funciton
    for (entity, mut _unit, transform, sprite, _waypoint) in &mut query.iter() {
        let unit_pos = transform.translation();
        match &mouse_command {
            Some(MouseCommand::SingleSelect(pos)) | Some(MouseCommand::Action(pos)) => {
                let unit_clicked = is_position_within_sprite(pos, &unit_pos, sprite);
                if unit_clicked {
                    selection_targets.push(entity);
                }
            },
            Some(MouseCommand::DragSelect{start, end}) => {
                if is_translation_within_box(&unit_pos, &start, &end) {
                    selection_targets.push(entity);
                }
            },
            None => (),
        }
    }

    match &mouse_command {
        // perform selections
        Some(MouseCommand::SingleSelect(_)) | Some(MouseCommand::DragSelect{start: _, end: _}) => {
            for (entity, mut unit, _transform, _sprite, _waypoint) in &mut query.iter() {
                if selection_targets.contains(&entity) {
                    if state.is_toggle_select_on {
                        unit.invert_select();
                    } else {
                        unit.select();
                    }
                } else {
                    // don't unselect units that weren't clicked on if multi-select or toggle-select are enabled
                    if !(state.is_multi_select_on || state.is_toggle_select_on) {
                        unit.deselect();
                    }
                }
            }
        },
        Some(MouseCommand::Action(pos)) => {
            let mut cmd = if let Some(target) = selection_targets.into_iter().next() {
                UnitCommands::AttackSlow(target)
            } else {
                UnitCommands::MoveSlow(pos.clone())
            };
            if is_double_click {
                cmd = cmd.to_fast();
            }
            new_commands.push(cmd);
        },
        None => (),
    }

    // send new commands to selected units
    // is it gross iterating over the query twice in one function?
    for (_entity, mut unit, _transform, _sprite, mut _waypoint) in &mut query.iter() {
        if unit.is_selected() {
            for cmd in new_commands.clone() {
                log::info!("Assigning {:?} command", cmd);
                unit.process_command(cmd.clone());
            }
        }
    }
}

struct CursorState {
    cursor: EventReader<CursorMoved>,
    // need to identify the main camera
    camera_e: Entity,
    last_pos: XyPos,
}

/// bevy doesn't provide a way of getting engine coordinates from the cursor, so this implementation stores it
/// in a resource so that it can be accesed by the input system
fn cursor_system(
    mut state: ResMut<CursorState>,
    ev_cursor: Res<Events<CursorMoved>>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera components
    q_camera: Query<&Transform>,
) {
    let camera_transform = q_camera.get::<Transform>(state.camera_e).expect("Camera Pos");

    for ev in state.cursor.iter(&ev_cursor) {
        // get the size of the window that the event is for
        let wnd = wnds.get(ev.id).expect("Window");
        let size = Vec2::new(wnd.width as f32, wnd.height as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = ev.position - size / 2.0;

        // apply the camera transform
        let pos_wld = camera_transform.value().clone() * p.extend(0.0).extend(1.0);

        *state.last_pos.x_mut() = pos_wld.x();
        *state.last_pos.y_mut() = pos_wld.y();
    }
}

// for each unit, calculates the position of its next waypoint
fn unit_waypoint_system(
    mut unit_query: Query<(&Unit, &mut Waypoint)>,
    target_query: Query<&Transform>,
) {
    for (unit, mut waypoint) in &mut unit_query.iter() {
        match &unit.current_command {
            UnitCurrentCommand::AttackSlow(target) | UnitCurrentCommand::AttackFast(target) => {
                let target_translation = target_query
                    .get::<Transform>(target.clone())
                    .expect("Target translation")
                    .translation();
                *waypoint =
                    Waypoint::Position((target_translation.x(), target_translation.y()).into());
            }
            UnitCurrentCommand::MoveSlow(wp) | UnitCurrentCommand::MoveFast(wp) => {
                // TODO this is unnessecary, but maybe its where its where we put in some pathfinding to determine the next step?
                *waypoint = Waypoint::Position(wp.clone());
            }
            UnitCurrentCommand::None_ => {}
        }
    }
}



// TODO have a separate component for waypoint position for all command types
// that is updated in a separate system, so its calculated separately from the unit movement system
// so we don't run into unique borrow issues
fn unit_movement_system(
    time: Res<Time>,
    mut bodies: ResMut<RigidBodySet>,
    mut colliders: ResMut<ColliderSet>,
    mut unit_query: Query<(&mut Unit, &mut Transform, &mut RigidBodyHandleComponent, &mut ColliderHandleComponent, &Waypoint)>,
) {
    for (mut unit, mut transform, body_handle, collider_handle,  waypoint) in &mut unit_query.iter() {
        let translation = transform.translation_mut();

        // TODO remove transform here, use rigid body pos
        let unit_pos: XyPos = (translation.x(), translation.y()).into();

        let mut body = bodies.get_mut(body_handle.handle()).expect("body");
        let mut collider = colliders.get_mut(collider_handle.handle()).expect("collider");

        // if the unit is going somewhere
        if let Some(dest) = match &unit.current_command {
            UnitCurrentCommand::AttackSlow(_) | UnitCurrentCommand::AttackFast(_) => {
                if let Waypoint::Position(xy) = waypoint {
                    Some(xy)
                } else {
                    log::error!("attack command without a waypoint!");
                    None
                }
            }
            UnitCurrentCommand::MoveSlow(_) | UnitCurrentCommand::MoveFast(_) => {
                if let Waypoint::Position(xy) = waypoint {
                    Some(xy)
                } else {
                    log::error!("attack command without a waypoint!");
                    None
                }
            }
            UnitCurrentCommand::None_ => None,
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
                let pos = Isometry::translation(
                    dest.x(),
                    dest.y(),
                );
                body.set_position(pos);
                collider.set_position_debug(pos);
                log::debug!("reached destination");
                unit.process_command(UnitCommands::Stop);
            }
        }
    }
}
