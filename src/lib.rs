
use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

#[derive(Clone)]
pub enum Waypoint {
    Position(XyPos),
    Unit,
}


pub type XyPos = Vec2;

pub enum UnitState {
    Idle,
    Firing,
    Melee,
    MovingFast(Waypoint),
    MovingSlow(Waypoint),
}

impl UnitState {
    pub fn display_text(&self) -> &str {
        match self {
            UnitState::Idle => "I",
            UnitState::Firing => "F",
            UnitState::Melee => "M",
            UnitState::MovingFast(_) => "R",
            UnitState::MovingSlow(_) => "W",
        }
    }
}

#[derive(Clone)]
pub enum UnitCommands {
    AttackFast,
    AttackSlow,
    MoveFast(XyPos),
    MoveSlow(XyPos),
    ToggleSpeed,
    Stop,
}


pub struct Unit {
    pub state: UnitState,
    max_speed: f32,
    is_selected: bool,
}

impl Unit {
    pub fn process_command(&mut self, cmd: UnitCommands) {
        use UnitCommands::*;
        match cmd {
            AttackFast | AttackSlow => unimplemented!(),
            Stop => self.state = UnitState::Idle,
            MoveFast(pos) => self.state = UnitState::MovingSlow(Waypoint::Position(pos)),
            MoveSlow(pos) => self.state = UnitState::MovingFast(Waypoint::Position(pos)),
            ToggleSpeed => {
                match &self.state {
                    UnitState::MovingFast(wp) => self.state = UnitState::MovingSlow(wp.clone()),
                    UnitState::MovingSlow(wp) => self.state = UnitState::MovingFast(wp.clone()),
                    _ => (),
                }
            }
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

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            state: UnitState::Idle,
            max_speed: 5.0,
            is_selected: false,
        }
    }
}