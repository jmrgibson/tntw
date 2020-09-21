#![allow(dead_code)]

use bevy::{
    prelude::*,
    render::pass::ClearColor,
};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

use tntw::{UnitCurrentCommand, UnitState, UnitCommands, Unit, XyPos, Waypoint};



fn main() {
    env_logger::init();
    App::build()
        .add_default_plugins()
        // .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .init_resource::<InputState>()
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_startup_system(setup.system())
        .add_system(unit_movement_system.system())
        .add_system(cursor_system.system())
        .add_system(input_system.system())
        // .add_system(ball_collision_system.system())
        // .add_system(ball_movement_system.system())
        // .add_system(scoreboard_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())

        // units
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .with(Unit::default())
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .with(Unit::default())
        // .with(Waypoint::default())
        ;


    // set up cursor tracker
    let camera = Camera2dComponents::default();
    let e = commands.spawn(camera).current_entity().unwrap();
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


struct SelectionMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
}

impl FromResources for SelectionMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        SelectionMaterials {
            normal: materials.add(Color::rgb(0.02, 0.02, 0.02).into()),
            hovered: materials.add(Color::rgb(0.05, 0.05, 0.05).into()),
            selected: materials.add(Color::rgb(0.1, 0.5, 0.1).into()),
        }
    }
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
    /// should this be an option?
    drag_select_start: XyPos,
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
            drag_select_start: (0.0, 0.0).into(),
        }
    }
}

/// TODO handle rotation, does this also handle dynamic sprite sizing?
/// TODO there is most certainly a better way of doing this math
fn is_position_within_sprite(position_to_check: XyPos, sprite_position: &Vec3, sprite: &Sprite) -> bool {
    position_to_check.x() < (sprite_position.x() + sprite.size.x()) && position_to_check.x() > (sprite_position.x() - sprite.size.x()) && 
    position_to_check.y() < (sprite_position.y() + sprite.size.y()) && position_to_check.y() > (sprite_position.y() - sprite.size.y()) 
}

fn input_system(
    mut state: ResMut<InputState>,
    cursor: Res<CursorState>,
    ev_keys: Res<Events<KeyboardInput>>,
    // ev_cursor: Res<Events<CursorMoved>>,
    // ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    // ev_scroll: Res<Events<MouseWheel>>,
    mut query: Query<(Entity, &mut Unit, &Transform, &Sprite)>,
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

    let mut mouse_command: Option<MouseButton> = None;
    let mut mouse_pos: Option<XyPos> = None;

    // get position and left/right of mouse button
    for ev in state.mousebtn.iter(&ev_mousebtn) {
        if ev.state.is_pressed() {
            // process on click
        } else {
            // process on release

            // get last cursor position 
            let position = cursor.last_pos.clone();
            mouse_pos.replace(position);

            if ev.button == MouseButton::Right {
                mouse_command.replace(MouseButton::Right);
            } else if ev.button == MouseButton::Left  {
                mouse_command.replace(MouseButton::Left);
            } else {
                // don't care about other buttons
                // TODO use an actual command enum
            }
        }
    }
     
    let mut any_unit_clicked: Option<Entity> = None;
    
    // determine if the mouse action was on a unit or not
    if let Some(selection_pos) = mouse_pos {
        for (entity, mut unit, transform, sprite) in &mut query.iter() {
            let unit_pos = transform.translation();

            let unit_clicked = is_position_within_sprite(selection_pos, &unit_pos, sprite);
            if unit_clicked {
                any_unit_clicked.replace(entity);
            }

            // handle select/deselect
            if let Some(command) = mouse_command {
                if command == MouseButton::Left {
                    if unit_clicked {
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
            }
        }
    }

   match (mouse_command, mouse_pos) {
       (Some(MouseButton::Left), _) => {
           // selection is handled in the query loop because life is complicated
       },
       (Some(MouseButton::Right), Some(mouse_pos)) => {
           if let Some(target) = any_unit_clicked {
                log::debug!("assigning attack target");
                new_commands.push(UnitCommands::AttackSlow(target));
            } else {
                log::debug!("assigning move waypoint");
                new_commands.push(UnitCommands::MoveSlow(mouse_pos));
           }
       },
       _ => (),
   }

    // send new commands to selected units
    // is it gross iterating over the query twice in one function?
    for (_entity, mut unit, _transform, _sprite) in &mut query.iter() {
        if unit.is_selected() {
            for cmd in new_commands.clone() {
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
    q_camera: Query<&Transform>
) {
    let camera_transform = q_camera.get::<Transform>(state.camera_e).unwrap();

    for ev in state.cursor.iter(&ev_cursor) {
        // get the size of the window that the event is for
        let wnd = wnds.get(ev.id).unwrap();
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


// TODO have a separate component for waypoint position for all command types
// that is updated in a separate system, so its calculated separately from the unit movement system
// so we don't run into unique borrow issues
fn unit_movement_system(
    time: Res<Time>,
    mut unit_query: Query<(&mut Unit, &mut Transform)>,
    target_query: Query<&Transform>,
) {
    // log::debug!("here?");
    for (mut unit, mut transform) in &mut unit_query.iter() {
        let translation = transform.translation_mut();
        let unit_pos: XyPos = (translation.x(), translation.y()).into();

        // if the unit is going somewhere
        if let Some(relative_position) = match &unit.current_command {
                UnitCurrentCommand::AttackSlow(target) | UnitCurrentCommand::AttackFast(target) => {
                    // panics at runtime becaue transform is already borrowed uniquely 
                    let target_translation = target_query.get::<Transform>(target.clone()).unwrap().translation();
                    let target_pos: XyPos = (target_translation.x(), target_translation.y()).into();
                    Some(target_pos - unit_pos)
                },     
                UnitCurrentCommand::MoveSlow(waypoint) | UnitCurrentCommand::MoveFast(waypoint) => {
                    Some(waypoint.clone() - unit_pos)
                },
                UnitCurrentCommand::None_ => {
                    None
                },
            }
        {
            let unit_distance = unit.current_speed() * time.delta_seconds;
        
            // using length_squared() for totally premature optimization
            let rel_distance_sq = relative_position.length_squared();
    
            // if we need to keep moving
            if unit_distance.powi(2) < rel_distance_sq {
                // get direction
                let direction = relative_position.normalize();
    
                // perform translation
                *translation.x_mut() = translation.x() + (direction.x() * unit_distance);
                *translation.y_mut() = translation.y() + (direction.y() * unit_distance);
            } else {
                // can reach destination, set position to waypoint, transition to idle
                *translation.x_mut() = translation.x() + relative_position.x();
                *translation.y_mut() = translation.y() + relative_position.y();
                log::debug!("reached destination");
                unit.process_command(UnitCommands::Stop);
            }
        }
    }
}
