use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

use tntw::{Waypoint, UnitState, UnitCommands, Unit, XyPos};



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
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            keys: EventReader::default(),
            cursor: EventReader::default(),
            motion: EventReader::default(),
            mousebtn: EventReader::default(),
            scroll: EventReader::default(),
        }
    }
}

fn input_system(
    mut state: ResMut<InputState>,
    cursor: Res<CursorState>,
    ev_keys: Res<Events<KeyboardInput>>,
    ev_cursor: Res<Events<CursorMoved>>,
    ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    ev_scroll: Res<Events<MouseWheel>>,
    mut query: Query<(&mut Unit, &Transform, &Sprite)>,
) {
    let mut new_commands: Vec<UnitCommands> = Vec::new();

    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            eprintln!("Just pressed key: {:?}", ev.key_code);
            if let Some(key) = ev.key_code {
                match key {
                    KeyCode::S => new_commands.push(UnitCommands::Stop), 
                    KeyCode::R => new_commands.push(UnitCommands::ToggleSpeed), 
                    _ => (),
                };

            }
        } else {
            // on release
            
        }
    }

    let mut maybe_selection_pos: Option<XyPos> = None;

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
                new_commands.push(UnitCommands::MoveSlow(position));
            } else if ev.button == MouseButton::Left  {
                log::trace!("Left click");
                maybe_selection_pos.replace(position);
            }
        }
    }

    for (mut unit, transform, sprite) in &mut query.iter() {
        if let Some(selection_pos) = maybe_selection_pos {
            let unit_pos = transform.translation();
            // check selection position is within unit center + sprite size bounds
            // TODO handle rotation, does this also handle dynamic sizing?
            // TODO there is most certainly a better way of doing this math
            if selection_pos.x() < (unit_pos.x() + sprite.size.x()) && selection_pos.x() > (unit_pos.x() - sprite.size.x()) && 
                selection_pos.y() < (unit_pos.y() + sprite.size.y()) && selection_pos.y() > (unit_pos.y() - sprite.size.y()) {
                unit.select();
            } else {
                unit.deselect();
            }
        }


        // send new commands to units
        for cmd in new_commands.clone() {
            if unit.is_selected() {
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

fn selection_system(
    button_materials: Res<SelectionMaterials>,
    mut interaction_query: Query<(
        &Button,
        Mutated<Interaction>,
        &mut Handle<ColorMaterial>,
        &mut Unit,
        &Children,
    )>,
    text_query: Query<&mut Text>,
) {
    for (_button, interaction, mut material, mut unit, children) in &mut interaction_query.iter() {
        let mut text = text_query.get_mut::<Text>(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                unit.select();
            }
            Interaction::Hovered => {
                text.value = "Select?".to_string();
                *material = button_materials.hovered;
            }
            Interaction::None => (),
        }
    }
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
        // paddle
        // .spawn(SpriteComponents {
        //     material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
        //     transform: Transform::from_translation(Vec3::new(0.0, -215.0, 0.0)),
        //     sprite: Sprite::new(Vec2::new(120.0, 30.0)),
        //     ..Default::default()
        // })
        // .with(Paddle { speed: 500.0 })
        // .with(Collider::Solid)

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
        // .with(ButtonComponents {
        //         style: Style {
        //             size: Size::new(Val::Px(150.0), Val::Px(65.0)),
        //             // center button
        //             margin: Rect::all(Val::Auto),
        //             // horizontally center child text
        //             justify_content: JustifyContent::Center,
        //             // vertically center child text
        //             align_items: AlignItems::Center,
        //             ..Default::default()
        //         },
        //         material: materials.unselected,
        //         ..Default::default()
        //     }
        // )

        // scoreboard
        // .spawn(TextComponents {
        //     text: Text {
        //         font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
        //         value: "Score:".to_string(),
        //         style: TextStyle {
        //             color: Color::rgb(0.2, 0.2, 0.8),
        //             font_size: 40.0,
        //         },
        //     },
        //     style: Style {
                // position_type: PositionType::Absolute,
        //         position: Rect {
        //             top: Val::Px(5.0),
        //             left: Val::Px(5.0),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // })
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

        // if the unit is going somewhere
        if let Some(relative_position) = unit.pos_rel_to_waypoint(translation){
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
                unit.state = UnitState::Idle;
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
