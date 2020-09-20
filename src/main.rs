use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

fn main() {
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

type XyPos = Vec2;

enum UnitState {
    Idle,
    Firing,
    Melee,
    MovingFast,
    MovingSlow(Waypoint),
}

#[derive(Clone)]
enum UnitCommands {
    AttackFast,
    AttackSlow,
    MoveFast(XyPos),
    MoveSlow(XyPos),
    Stop,
}

pub enum Waypoint {
    Position(XyPos),
    Unit,
}

struct Unit {
    state: UnitState,
    max_speed: f32,
    is_selected: bool,
}

impl Unit {
    fn assign_waypoint() {

    }

    fn process_command(&mut self, cmd: UnitCommands) {
        use UnitCommands::*;
        match cmd {
            AttackFast | AttackSlow => {},
            Stop => self.state = UnitState::Idle,
            MoveFast(pos) | MoveSlow(pos) => self.state = UnitState::MovingSlow(Waypoint::Position(pos)),
        }
    }
    fn select(&mut self) {
        self.is_selected = true;
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            state: UnitState::MovingSlow(Waypoint::Position((50.0, 50.0).into())),
            max_speed: 50.0,
            // TODO implement selection
            is_selected: true,
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
    mut query: Query<&mut Unit>,
) {
    let mut new_command: Option<UnitCommands> = None;

    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            eprintln!("Just pressed key: {:?}", ev.key_code);
            if let Some(key) = ev.key_code {
                if key == KeyCode::S {
                    new_command.replace(UnitCommands::Stop);
                }
            }
        } else {
            
        }
    }

    // Absolute cursor position (in window coordinates)
    // for ev in state.cursor.iter(&ev_cursor) {
        // eprintln!("Cursor at: {}", ev.position);
    // }

    // Relative mouse motion
    // for ev in state.motion.iter(&ev_motion) {
        // eprintln!("Mouse moved {} pixels", ev.delta);
    // }



    // Mouse buttons
    for ev in state.mousebtn.iter(&ev_mousebtn) {
        if ev.state.is_pressed() {
            // process select
            // eprintln!("Just pressed mouse button: {:?}", ev.button);
        } else {
            // eprintln!("Just released mouse button: {:?}", ev.button);
            // process on release
            // TODO handle left/right click

            // get last cursor position 
            let position = cursor.last_pos.clone();

            new_command.replace(UnitCommands::MoveSlow(position));
        }
    }

    // scrolling (mouse wheel, touchpad, etc.)
    for ev in state.scroll.iter(&ev_scroll) {
        eprintln!("Scrolled vertically by {} and horizontally by {}.", ev.y, ev.x);
    }

    if let Some(cmd) = new_command {
        for mut unit in &mut query.iter() {
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
/// so that it can be accesed by the input system
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

// fn command_system(
//     mut interaction_query: Query<(
//         &mut Unit
//     )>
// ) {

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
        // paddle
        // .spawn(SpriteComponents {
        //     material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
        //     transform: Transform::from_translation(Vec3::new(0.0, -215.0, 0.0)),
        //     sprite: Sprite::new(Vec2::new(120.0, 30.0)),
        //     ..Default::default()
        // })
        // .with(Paddle { speed: 500.0 })
        // .with(Collider::Solid)

        // unit
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -50.0, 1.0)),
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
        //         material: materials.unse;ec,
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
        //         position_type: PositionType::Absolute,
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

    // // Add bricks
    // let brick_rows = 4;
    // let brick_columns = 5;
    // let brick_spacing = 20.0;
    // let brick_size = Vec2::new(150.0, 30.0);
    // let bricks_width = brick_columns as f32 * (brick_size.x() + brick_spacing) - brick_spacing;
    // // center the bricks and move them up a bit
    // let bricks_offset = Vec3::new(-(bricks_width - brick_size.x()) / 2.0, 100.0, 0.0);

    // for row in 0..brick_rows {
    //     let y_position = row as f32 * (brick_size.y() + brick_spacing);
    //     for column in 0..brick_columns {
    //         let brick_position = Vec3::new(
    //             column as f32 * (brick_size.x() + brick_spacing),
    //             y_position,
    //             0.0,
    //         ) + bricks_offset;
    //         commands
    //             // brick
    //             .spawn(SpriteComponents {
    //                 material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
    //                 sprite: Sprite::new(brick_size),
    //                 transform: Transform::from_translation(brick_position),
    //                 ..Default::default()
    //             })
    //             .with(Collider::Scorable);
    //     }
    // }
}

fn unit_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Unit, &mut Transform)>,
) {
    for (mut unit, mut transform) in &mut query.iter() {
        match &mut unit.state {
            UnitState::MovingSlow(waypoint) => {
                if let Waypoint::Position(wpos) = waypoint {
                    // move unit (with its own translation) 
                    let translation = transform.translation_mut();
                    // get direction and normalize
                    let pos: XyPos = (translation.x(), translation.y()).into();
                    let direction =  (wpos.clone() - pos).normalize();
                    // translation
                    *translation.x_mut() = translation.x() + direction.x();
                    *translation.y_mut() = translation.y() + direction.y();
                }
            },
            _ => (),
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
