
use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

const WALKING_SPEED_FACTOR: f32 = 0.5;

pub type XyPos = Vec2;

pub struct Unit {
    pub current_command: UnitCurrentCommand,
    max_speed: f32,
    is_selected: bool,
}

pub enum Waypoint {
    None,
    Position(XyPos),
    Unit(Entity),
}

/// Intended for UI display
pub enum UnitState {
    Idle,
    Firing,
    Melee,
    MovingFast,
    MovingSlow,
}

impl UnitState {
    pub fn display_text(&self) -> &str {
        match self {
            UnitState::Idle => "I",
            UnitState::Firing => "F",
            UnitState::Melee => "M",
            UnitState::MovingFast => "R",
            UnitState::MovingSlow => "W",
        }
    }
}

#[derive(Clone)]
pub enum UnitCurrentCommand {
    AttackFast(Entity),
    AttackSlow(Entity),
    MoveFast(XyPos),
    MoveSlow(XyPos),
    None_,
}

#[derive(Clone)]
pub enum UnitCommands {
    AttackFast(Entity),
    AttackSlow(Entity),
    MoveFast(XyPos),
    MoveSlow(XyPos),
    ToggleSpeed,
    Stop,
}

impl Unit {
    pub fn process_command(&mut self, cmd: UnitCommands) {
        use UnitCommands::*;
        match cmd {
            AttackFast(target) => {
                self.current_command = UnitCurrentCommand::AttackFast(target);
            },
            AttackSlow(target) => {
                self.current_command = UnitCurrentCommand::AttackSlow(target);
            },
            MoveFast(pos) => {
                self.current_command = UnitCurrentCommand::MoveFast(pos);
            },
            MoveSlow(pos) => {
                self.current_command = UnitCurrentCommand::MoveSlow(pos)
            },
            Stop => {
                self.current_command = UnitCurrentCommand::None_;
            },
            ToggleSpeed => {
                match &self.current_command {
                    UnitCurrentCommand::MoveFast(wp) => self.current_command = UnitCurrentCommand::MoveSlow(wp.clone()),
                    UnitCurrentCommand::MoveSlow(wp) => self.current_command = UnitCurrentCommand::MoveFast(wp.clone()),
                    _ => (),
                }
            }
        }
    }

    pub fn state(&self) -> UnitState {
        use UnitCurrentCommand::*;
        match self.current_command {
            AttackFast(_) | MoveFast(_) => UnitState::MovingFast,
            AttackSlow(_) | MoveSlow(_) => UnitState::MovingSlow,
            None_ => UnitState::Idle,
        }
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
        match self.current_command {
            UnitCurrentCommand::MoveSlow(_) => self.max_speed * WALKING_SPEED_FACTOR,
            UnitCurrentCommand::MoveFast(_) => self.max_speed,
            _ => 0.0,
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            current_command: UnitCurrentCommand::None_,
            max_speed: 50.0,
            is_selected: false,
        }
    }
}

impl Default for Waypoint {
    fn default() -> Self {
        Waypoint::None
    }
}