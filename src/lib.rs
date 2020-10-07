#![deny(unreachable_patterns)]
use bevy::prelude::*;

use crate::game_speed::{GameSpeed, GameSpeedRequest};
use crate::physics::ContactType;
use crate::teams::*;

pub mod combat;
pub mod game_speed;
pub mod physics;
pub mod teams;
pub mod ui;
pub mod units;
pub mod user_input;

const WALKING_SPEED_FACTOR: f32 = 0.5;
const MAX_HP: f32 = 100.0;

pub type XyPos = Vec2;

pub struct DebugTimer(pub Timer);

#[derive(Default)]
pub struct UnitInteractionState {
    pub event_reader: EventReader<UnitInteractionEvent>,
}

#[derive(Clone, Copy, Debug)]
pub enum UnitInteractionEvent {
    Proximity(ContactType),
    Ui(Entity, UnitUiCommand),
    UnitDied(Entity),
    UnitWaypointReached(Entity),
}

pub enum MissileType {
    Bow,
    Javelin,
    Sling,
}

pub struct MissileStats {
    pub max_ammunition: usize,
    pub current_ammunition: usize,
    pub range: f32,
    pub type_: MissileType,
}

pub enum MissileWeapon {
    Primary(MissileStats),
    Secondary(MissileStats),
    None,
}

pub struct UnitComponent {
    pub current_command: UnitUserCommand,
    pub state: UnitState,
    max_speed: f32,
    is_selected: bool,
    pub team: TeamId,
    pub missile_weapon: MissileWeapon,
    pub unit_type: UnitType,
    pub is_running: bool,
    /// "guard mode" determines if the current unit will persue fleeing units if they
    /// attempt to disengage melee
    pub guard_mode_enabled: bool,
    /// "fire at will" determines if the unit will automatically use ranged projectiles
    /// at any enemy unit that enters its firing range
    pub fire_at_will: bool,
    pub remaining_ammo: usize,
}

pub struct HealthComponent {
    current_health: f32,
    max_health: f32,
}

pub enum WaypointComponent {
    None,
    Position(XyPos),
}

#[derive(PartialEq)]
pub enum AttackType {
    Melee,
    Ranged,
}

#[derive(PartialEq, Clone, Copy)]
pub enum UnitType {
    MeleeCalvary,
    ShockCalvary,
    MissileCalvary,
    MeleeInfantry,
    PikeInfantry,
    ShockInfantry,
    SpearInfantry,
    MissileInfantry,
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
    AttackMelee(Entity),
    AttackMissile(Entity),
    Move(XyPos),
    None_,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnitState {
    Idle,
    /// optional entity is the unit it is fighting.
    /// is Some if target is still alive, None if target died 
    Firing(Option<Entity>),
    /// Damn horse archers complicating things
    FiringAndMoving(Option<Entity>),
    Melee(Option<Entity>),
    Moving,
}

impl UnitState {
    /// is Some if target is still alive, None if target died 
    pub fn current_actively_fighting(&self) -> Option<Entity> {
        if let UnitState::Melee(e) | UnitState::Firing(e) | UnitState::FiringAndMoving(e) = self {
            e.map(|e| e.clone())
        } else {
            None
        }
    }

    pub fn clear_target(&mut self) {
        match self {
            UnitState::Melee(_) => *self = UnitState::Melee(None),
            UnitState::Firing(_) => *self = UnitState::Firing(None),
            UnitState::FiringAndMoving(_) => *self = UnitState::FiringAndMoving(None),
            _ => (),
        }
    }
}

/// possible actions given to the unit by the user
#[derive(Clone, Copy, Debug)]
pub enum UnitUiCommand {
    Attack(Entity, UnitUiSpeedCommand),
    Move(XyPos, UnitUiSpeedCommand),
    ToggleSpeed,
    ToggleGuardMode,
    ToggleFireAtWill,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitUiSpeedCommand {
    Run,
    Walk,
}

impl UnitComponent {
    pub fn ui_state(&self) -> UnitUiState {
        match &self.state {
            UnitState::Melee(_) => UnitUiState::Melee,
            UnitState::Moving => {
                if self.is_running {
                    UnitUiState::MovingFast
                } else {
                    UnitUiState::MovingSlow
                }
            }
            UnitState::Idle => UnitUiState::Idle,
            UnitState::Firing(_) | UnitState::FiringAndMoving(_) => UnitUiState::Firing,
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
        if self.is_running {
            self.max_speed
        } else {
            self.max_speed * WALKING_SPEED_FACTOR
        }
    }

    pub fn is_missile_attack_available(&self) -> bool {
        if let MissileWeapon::Primary(stats) | MissileWeapon::Secondary(stats) =
            &self.missile_weapon
        {
            stats.current_ammunition > 0
        } else {
            false
        }
    }

    pub fn default_from_type(unit_type: UnitType, team: TeamId) -> UnitComponent {
        match unit_type {
            UnitType::MeleeInfantry => UnitComponent {
                max_speed: 50.0,
                unit_type,
                team,
                ..UnitComponent::default()
            },
            UnitType::MissileInfantry => UnitComponent {
                max_speed: 80.0,
                missile_weapon: MissileWeapon::Primary(MissileStats {
                    max_ammunition: 500,
                    current_ammunition: 500,
                    range: 100.0,
                    type_: MissileType::Bow,
                }),
                unit_type,
                team,
                ..UnitComponent::default()
            },
            _ => unimplemented!(),
        }
    }

    pub fn primary_attack_type(&self) -> AttackType {
        match self.unit_type {
            UnitType::MissileInfantry | UnitType::MissileCalvary => AttackType::Ranged,
            _ => AttackType::Melee,
        }
    }

    pub fn can_fire_while_moving(&self) -> bool {
        self.unit_type == UnitType::MissileCalvary
    }
}

impl Default for UnitComponent {
    fn default() -> Self {
        UnitComponent {
            current_command: UnitUserCommand::None_,
            state: UnitState::Idle,
            max_speed: 50.0,
            is_selected: false,
            is_running: false,
            guard_mode_enabled: false,
            unit_type: UnitType::MeleeInfantry,
            remaining_ammo: 10,
            fire_at_will: true,
            team: 0,
            missile_weapon: MissileWeapon::None,
        }
    }
}

impl Default for WaypointComponent {
    fn default() -> Self {
        WaypointComponent::None
    }
}

impl Default for HealthComponent {
    fn default() -> Self {
        HealthComponent {
            max_health: MAX_HP,
            current_health: MAX_HP,
        }
    }
}
