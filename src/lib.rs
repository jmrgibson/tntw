
use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

const WALKING_SPEED_FACTOR: f32 = 0.5;

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

    pub fn current_speed(&self) -> f32 {
        match self.state {
            UnitState::MovingSlow(_) => self.max_speed * WALKING_SPEED_FACTOR,
            UnitState::MovingFast(_) => self.max_speed,
            _ => 0.0,
        }
    }

    /// returns the relative translation of a given position from the units waypoint
    pub fn pos_rel_to_waypoint(&self, current_pos: &Vec4) -> Option<Vec2> {
        match &self.state {
            UnitState::MovingSlow(waypoint) | UnitState::MovingFast(waypoint) => {
                if let Waypoint::Position(wpos) = waypoint {
                    // get direction and normalize
                    let pos: XyPos = (current_pos.x(), current_pos.y()).into();
                    Some(wpos.clone() - pos)
                } else {
                    unimplemented!();
                }
            },
            UnitState::Idle => {
                None
            }
            _ => unimplemented!()
        }
    }

}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            state: UnitState::Idle,
            max_speed: 50.0,
            is_selected: false,
        }
    }
}