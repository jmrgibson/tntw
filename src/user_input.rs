use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_input::keyboard::*;
use bevy_input::mouse::*;

use crate::*;

pub enum MouseCommand {
    SingleSelect(XyPos),
    DragSelect {
        start: XyPos,
        end: XyPos,
    },
    /// attack/move
    Action(XyPos),
}

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);
const DRAG_SELECT_MIN_BOX: f32 = 100.0;

pub struct InputState {
    keys: EventReader<KeyboardInput>,
    mousebtn: EventReader<MouseButtonInput>,
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
            mousebtn: EventReader::default(),
            is_multi_select_on: false,
            is_toggle_select_on: false,
            last_mouse_action: None,
            drag_select_start: None,
        }
    }
}

pub fn input_system(
    mut engine_commands: Commands,
    mut state: ResMut<InputState>,
    cursor: Res<CursorState>,
    ev_keys: Res<Events<KeyboardInput>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    mut unit_events: ResMut<Events<UnitInteractionEvent>>,
    mut query: Query<(
        Entity,
        &mut UnitComponent,
        &Transform,
        &Sprite,
        &mut WaypointComponent,
    )>,
) {
    let mut ui_commands: Vec<UnitUiCommand> = Vec::new();

    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            // on press
            if let Some(key) = ev.key_code {
                log::trace!("pressed {:?}", key);
                match key {
                    KeyCode::S => ui_commands.push(UnitUiCommand::Stop),
                    KeyCode::R => ui_commands.push(UnitUiCommand::ToggleSpeed),
                    KeyCode::G => ui_commands.push(UnitUiCommand::ToggleGuardMode),
                    KeyCode::F => ui_commands.push(UnitUiCommand::ToggleFireAtWill),
                    KeyCode::Tab => {
                        // remember, must be tuple here!
                        engine_commands.spawn((GameSpeedRequest::TogglePause,));
                    }
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
                    if (start.x - mouse_position.x).abs() > DRAG_SELECT_MIN_BOX
                        && (start.y - mouse_position.y).abs() > DRAG_SELECT_MIN_BOX
                    {
                        mouse_command.replace(MouseCommand::DragSelect {
                            start,
                            end: mouse_position,
                        });
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
    for (entity, mut _unit, transform, sprite, _waypoint) in query.iter_mut() {
        let unit_pos = transform.translation;
        match &mouse_command {
            Some(MouseCommand::SingleSelect(pos)) | Some(MouseCommand::Action(pos)) => {
                let unit_clicked = is_position_within_sprite(pos, &unit_pos, sprite);
                if unit_clicked {
                    selection_targets.push(entity);
                }
            }
            Some(MouseCommand::DragSelect { start, end }) => {
                if is_translation_within_box(&unit_pos, &start, &end) {
                    selection_targets.push(entity);
                }
            }
            None => (),
        }
    }

    match &mouse_command {
        // perform selections
        Some(MouseCommand::SingleSelect(_))
        | Some(MouseCommand::DragSelect { start: _, end: _ }) => {
            for (entity, mut unit, _transform, _sprite, _waypoint) in query.iter_mut() {
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
        }
        Some(MouseCommand::Action(pos)) => {
            let speed = if is_double_click {
                UnitUiSpeedCommand::Run
            } else {
                UnitUiSpeedCommand::Walk
            };
            let cmd = if let Some(target) = selection_targets.into_iter().next() {
                UnitUiCommand::Attack(target, speed)
            } else {
                UnitUiCommand::Move(pos.clone(), speed)
            };
            ui_commands.push(cmd);
        }
        None => (),
    }

    // send new commands to selected units
    // is it gross iterating over the query twice in one function?
    for (entity, unit, _transform, _sprite, mut _waypoint) in query.iter_mut() {
        if unit.is_selected() {
            for cmd in ui_commands.clone() {
                unit_events.send(UnitInteractionEvent::Ui(entity, cmd));
                log::info!("Assigning {:?} command", cmd);
            }
        }
    }
}

pub struct CursorState {
    pub cursor: EventReader<CursorMoved>,
    /// need to identify the main camera
    pub camera_e: Entity,
    pub last_pos: XyPos,
}

/// bevy doesn't provide a way of getting engine coordinates from the cursor, so this implementation stores it
/// in a resource so that it can be accesed by the input system
pub fn cursor_system(
    mut state: ResMut<CursorState>,
    ev_cursor: Res<Events<CursorMoved>>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera components
    q_camera: Query<&Transform>,
) {
    let camera_transform = q_camera
        .get_component::<Transform>(state.camera_e)
        .expect("Camera Pos");

    for ev in state.cursor.iter(&ev_cursor) {
        // get the size of the window that the event is for
        let wnd = wnds.get(ev.id).expect("Window");
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = ev.position - size / 2.0;

        // apply the camera transform
        let pos_wld = camera_transform.compute_matrix().clone() * p.extend(0.0).extend(1.0);

        state.last_pos.x = pos_wld.x;
        state.last_pos.y = pos_wld.y;
    }
}

/// TODO handle rotation, does this also handle dynamic sprite sizing?
/// TODO there is most certainly a better way of doing this math
fn is_position_within_sprite(
    position_to_check: &XyPos,
    sprite_position: &Vec3,
    sprite: &Sprite,
) -> bool {
    position_to_check.x < (sprite_position.x + sprite.size.x)
        && position_to_check.x > (sprite_position.x - sprite.size.x)
        && position_to_check.y < (sprite_position.y + sprite.size.y)
        && position_to_check.y > (sprite_position.y - sprite.size.y)
}

/// TODO  also fairly gross
fn is_translation_within_box(position_to_check: &Vec3, corner: &Vec2, end: &Vec2) -> bool {
    let in_x = if end.x - corner.x > 0.0 {
        corner.x < position_to_check.x && position_to_check.x < end.x
    } else {
        corner.x > position_to_check.x && position_to_check.x > end.x
    };
    let in_y = if end.y - corner.y > 0.0 {
        corner.y < position_to_check.y && position_to_check.y < end.y
    } else {
        corner.y > position_to_check.y && position_to_check.y > end.y
    };
    in_x && in_y
}
