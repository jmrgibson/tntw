#[deny(unreachable_patterns)]
use bevy::{
    prelude::*,
};

use crate::physics::ContactType;
use crate::teams::*;

pub mod combat;
pub mod physics;
pub mod teams;
pub mod ui;
pub mod units;

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
}

pub struct UnitComponent {
    pub current_command: UnitUserCommand,
    pub current_action: UnitCurrentAction,
    max_speed: f32,
    is_selected: bool,
    pub team: TeamId,
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

pub enum PrimaryAttackType {
    Melee,
    Ranged,
}

pub enum UnitType {
    MeleeCalvary,
    ShockCalvary,
    SkirmishCalvary,
    MeleeInfantry,
    PikeInfantry,
    ShockInfantry,
    SpearInfantry,
    SkirmishInfantry,
}

impl UnitType {
    pub fn primary_attack_type(&self) -> PrimaryAttackType {
        match self {
            UnitType::SkirmishInfantry | UnitType::SkirmishCalvary => PrimaryAttackType::Ranged,
            _ => PrimaryAttackType::Melee,
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub enum UnitCurrentAction {
    Idle,
    Firing(Entity),
    Melee(Entity),
    Moving,
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
        match &self.current_action {
            UnitCurrentAction::Melee(_) => UnitUiState::Melee,
            UnitCurrentAction::Moving => {
                if self.is_running {
                    UnitUiState::MovingFast
                } else {
                    UnitUiState::MovingSlow
                }
            }
            UnitCurrentAction::Idle => UnitUiState::Idle,
            UnitCurrentAction::Firing(_) => UnitUiState::Firing,
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

    pub fn default_from_type(unit_type: UnitType, team: TeamId) -> UnitComponent {
        match unit_type {
            UnitType::MeleeInfantry => UnitComponent {
                max_speed: 50.0,
                unit_type,
                team,
                ..UnitComponent::default()
            },
            UnitType::SkirmishInfantry => UnitComponent {
                max_speed: 80.0,
                unit_type,
                team,
                ..UnitComponent::default()
            },
            _ => unimplemented!(),
        }
    }
}

impl Default for UnitComponent {
    fn default() -> Self {
        UnitComponent {
            current_command: UnitUserCommand::None_,
            current_action: UnitCurrentAction::Moving,
            max_speed: 50.0,
            is_selected: false,
            is_running: false,
            guard_mode_enabled: false,
            unit_type: UnitType::MeleeInfantry,
            remaining_ammo: 10,
            fire_at_will: true,
            team: 0,
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
