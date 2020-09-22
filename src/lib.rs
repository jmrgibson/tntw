
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

#[derive(Clone, Debug)]
pub enum UnitCurrentCommand {
    AttackFast(Entity),
    AttackSlow(Entity),
    MoveFast(XyPos),
    MoveSlow(XyPos),
    None_,
}

impl UnitCurrentCommand {
}

#[derive(Clone, Debug)]
pub enum UnitCommands {
    AttackFast(Entity),
    AttackSlow(Entity),
    MoveFast(XyPos),
    MoveSlow(XyPos),
    ToggleSpeed,
    Stop,
}

impl UnitCommands {
    pub fn has_waypoint(&self) -> bool {
        use UnitCommands::*;
        match self {
            AttackFast(_) | AttackSlow(_) | MoveFast(_) | MoveSlow(_) => true,
            _ => false,
        }
    }
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
                    UnitCurrentCommand::AttackFast(wp) => self.current_command = UnitCurrentCommand::AttackSlow(wp.clone()),
                    UnitCurrentCommand::AttackSlow(wp) => self.current_command = UnitCurrentCommand::AttackFast(wp.clone()),
                    _ => (),
                }
            }
        }
        log::debug!("unit current command: {:?}", self.current_command);
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
        if self.is_walking() {
            self.max_speed * WALKING_SPEED_FACTOR
        } else {
            self.max_speed
        }
    }
    
    pub fn is_walking(&self) -> bool {
        use UnitCurrentCommand::*;
        match self.current_command {
            AttackSlow(_) | MoveSlow(_) => true,
            _ => false,
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