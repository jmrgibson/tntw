#![allow(dead_code)]

use bevy::{
    prelude::*,
    render::pass::ClearColor,
};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

use tntw::{UnitCurrentCommand, UnitState, UnitCommands, Unit, XyPos};



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
    ev_cursor: Res<Events<CursorMoved>>,
    ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    ev_scroll: Res<Events<MouseWheel>>,
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

    let mut mouse_command_and_pos: Option<(MouseButton, XyPos)> = None;

    // Mouse buttons
    for ev in state.mousebtn.iter(&ev_mousebtn) {
        if ev.state.is_pressed() {
            // process on click
            // eprintln!("Just pressed mouse button: {:?}", ev.button);
        } else {
            // eprintln!("Just released mouse button: {:?}", ev.button);
            // process on release

            // get last cursor position 
            let position = cursor.last_pos.clone();

            if ev.button == MouseButton::Right {
                log::trace!("Right click");
                mouse_command_and_pos.replace((MouseButton::Right, position));
            } else if ev.button == MouseButton::Left  {
                log::trace!("Left click");
                mouse_command_and_pos.replace((MouseButton::Left, position));
            }
        }
    }

    // 
    let mut any_unit_clicked: Option<(Entity, MouseButton)> = None;
    
    // new_commands.push(UnitCommands::MoveSlow(position));
    // determine if the mouse action was on a unit or not
    if let Some((command, selection_pos)) = mouse_command_and_pos {
        for (entity, mut unit, transform, sprite) in &mut query.iter() {
            let unit_pos = transform.translation();

            let unit_clicked = is_position_within_sprite(selection_pos, &unit_pos, sprite);
            if unit_clicked {
                any_unit_clicked.replace((entity, command));
            }

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

    if let Some((target, command)) = any_unit_clicked {
        match command {
            MouseButton::Left => {
                // selection is handled in the query loop because life is complicated
            },
            MouseButton::Right => {
                log::debug!("assigning attack target");
                new_commands.push(UnitCommands::AttackSlow(target));
            },
            _ => (),
        }
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

// fn selection_system(
//     button_materials: Res<SelectionMaterials>,
//     mut interaction_query: Query<(
//         &Button,
//         Mutated<Interaction>,
//         &mut Handle<ColorMaterial>,
//         &mut Unit,
//         &Children,
//     )>,
//     text_query: Query<&mut Text>,
// ) {
//     for (_button, interaction, mut material, mut unit, children) in &mut interaction_query.iter() {
//         let mut text = text_query.get_mut::<Text>(children[0]).unwrap();
//         match *interaction {
//             Interaction::Clicked => {
//                 unit.select();
//             }
//             Interaction::Hovered => {
//                 text.value = "Select?".to_string();
//                 *material = button_materials.hovered;
//             }
//             Interaction::None => (),
//         }
//     }
// }

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

 


fn unit_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Unit, &mut Transform)>,
) {
    for (mut unit, mut transform) in &mut query.iter() {
        let translation = transform.translation_mut();
        let unit_pos: XyPos = (translation.x(), translation.y()).into();

        // if the unit is going somewhere
        if let Some(relative_position) = match &unit.current_command {
                UnitCurrentCommand::AttackSlow(target) | UnitCurrentCommand::AttackFast(target) => {
                    let target_translation = query.get::<&Transform>(target.clone()).unwrap().translation();
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

// fn ball_movement_system(time: Res<Time>, mut ball_query: Query<(&Ball, &mut Transform)>) {
//     // clamp the timestep to stop the ball from escaping when the game starts
//     let delta_seconds = f32::min(0.2, time.delta_seconds);

//     for (ball, mut transform) in &mut ball_query.iter() {
//         transform.translate(ball.velocity * delta_seconds);
//     }
// }

// fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
//     for mut text in &mut query.iter() {
//         text.value = format!("Score: {}", scoreboard.score);
//     }
// }

// fn ball_collision_system(
//     mut commands: Commands,
//     mut scoreboard: ResMut<Scoreboard>,
//     mut ball_query: Query<(&mut Ball, &Transform, &Sprite)>,
//     mut collider_query: Query<(Entity, &Collider, &Transform, &Sprite)>,
// ) {
//     for (mut ball, ball_transform, sprite) in &mut ball_query.iter() {
//         let ball_size = sprite.size;
//         let velocity = &mut ball.velocity;

//         // check collision with walls
//         for (collider_entity, collider, transform, sprite) in &mut collider_query.iter() {
//             let collision = collide(
//                 ball_transform.translation(),
//                 ball_size,
//                 transform.translation(),
//                 sprite.size,
//             );
//             if let Some(collision) = collision {
//                 // scorable colliders should be despawned and increment the scoreboard on collision
//                 if let Collider::Scorable = *collider {
//                     scoreboard.score += 1;
//                     commands.despawn(collider_entity);
//                 }

//                 // reflect the ball when it collides
//                 let mut reflect_x = false;
//                 let mut reflect_y = false;

//                 // only reflect if the ball's velocity is going in the opposite direction of the collision
//                 match collision {
//                     Collision::Left => reflect_x = velocity.x() > 0.0,
//                     Collision::Right => reflect_x = velocity.x() < 0.0,
//                     Collision::Top => reflect_y = velocity.y() < 0.0,
//                     Collision::Bottom => reflect_y = velocity.y() > 0.0,
//                 }

//                 // reflect velocity on the x-axis if we hit something on the x-axis
//                 if reflect_x {
//                     *velocity.x_mut() = -velocity.x();
//                 }

//                 // reflect velocity on the y-axis if we hit something on the y-axis
//                 if reflect_y {
//                     *velocity.y_mut() = -velocity.y();
//                 }

//                 break;
//             }
//         }
//     }
// }
