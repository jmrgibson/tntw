#[deny(unreachable_patterns)]

use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

use crate::physics::ContactType;

pub mod physics;
pub mod ui;

const WALKING_SPEED_FACTOR: f32 = 0.5;

pub type XyPos = Vec2;

pub struct DebugTimer(pub Timer);

#[derive(Default)]
pub struct UnitInteractionState {
    pub event_reader: EventReader<UnitInteractionEvent>,
}

pub struct UnitInteractionEvent {
    pub interaction: physics::ContactType,
}

pub struct Unit {
    pub current_command: UnitUserCommand,
    pub current_action: UnitCurrentAction,
    max_speed: f32,
    is_selected: bool,
    is_running: bool,
    /// "guard mode" determines if the current unit will persue fleeing units if they 
    /// attempt to disengage melee
    guard_mode_enabled: bool,
}

pub enum Waypoint {
    None,
    Position(XyPos),
}

/// Intended for UI display
pub enum UnitUiState {
    Idle,
    Firing,
    Melee,
    MovingFast,
    MovingSlow,
}

/// the current command given to this unit by the user.
#[derive(Clone, Debug)]
pub enum UnitUserCommand {
    Attack(Entity),
    Move(XyPos),
    None_,
}

#[derive(Clone, Debug)]
pub enum UnitCurrentAction {
    Moving,
    Melee,
    Idle,
}

/// possible actions given to the unit by the user
#[derive(Clone, Debug)]
pub enum UnitUiCommands {
    Attack(Entity, UnitUiSpeedCommand),
    Move(XyPos, UnitUiSpeedCommand),
    ToggleSpeed,
    ToggleGuardMode,
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnitUiSpeedCommand {
    Run,
    Walk,
}

impl UnitUiCommands {
    pub fn has_waypoint(&self) -> bool {
        use UnitUiCommands::*;
        match self {
            Attack(_, _) | Move(_, _) => true,
            _ => false,
        }
    }

}

impl Unit {
    pub fn process_command(&mut self, cmd: UnitUiCommands) {
        use UnitUiCommands::*;
        match cmd {
            Attack(target, speed) => {
                self.current_command = UnitUserCommand::Attack(target);
                self.is_running = speed == UnitUiSpeedCommand::Run;
            },
            Move(pos, speed) => {
                self.current_command = UnitUserCommand::Move(pos);
                self.is_running = speed == UnitUiSpeedCommand::Run;
            },
            Stop => {
                self.current_command = UnitUserCommand::None_;
            },
            ToggleGuardMode => {
                self.guard_mode_enabled = !self.guard_mode_enabled;
            },
            ToggleSpeed => {
                self.is_running = !self.is_running;
            },
        }
        log::debug!("unit current command: {:?}", self.current_command);
    }

    pub fn ui_state(&self) -> UnitUiState {
        use UnitCurrentAction::*;
        match &self.current_action {
            Melee => UnitUiState::Melee,
            Moving => {
                if self.is_running {
                    UnitUiState::MovingFast 
                } else {
                    UnitUiState::MovingSlow
                }
            },
            Idle => UnitUiState::Idle,
        }
    }

    /// TODO
    pub fn is_same_team(&self, other: &Unit) -> bool {
        false
    }

    pub fn select(&mut self) {
        log::debug!("Unit selected");
        self.is_selected = true;
    }
    
    pub fn deselect(&mut self) {
        log::debug!("Unit deselected");
        self.is_selected = false;
    }

    pub fn invert_select(&mut self) {
        log::debug!("Unit invert selected");
        self.is_selected = !self.is_selected;
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    pub fn current_speed(&self) -> f32 {
        if self.is_running {
            self.max_speed
        } else {
            self.max_speed * WALKING_SPEED_FACTOR
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            current_command: UnitUserCommand::None_,
            current_action: UnitCurrentAction::Moving,
            max_speed: 50.0,
            is_selected: false,
            is_running: false,
            guard_mode_enabled: false,
        }
    }
}

impl Default for Waypoint {
    fn default() -> Self {
        Waypoint::None
    }
}