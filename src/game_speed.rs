use bevy::prelude::*;

/// A resource which stores the current game speed and elapsed game time
pub struct GameSpeed {
    game_speed: f32,
    elapsed_time: f32,
    delta: f32,
    is_paused: bool,
}

impl Default for GameSpeed {
    fn default() -> GameSpeed {
        GameSpeed {
            game_speed: 0.0,
            elapsed_time: 0.0,
            delta: 0.0,
            is_paused: false,
        }
    }
}

impl GameSpeed {
    /// Returns true if the game is currently paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn unpause(&mut self) {
        self.is_paused = false;
    }
}

/// Add this on a new entity (with no other components) to request a game speed change
#[derive(Debug)]
pub enum GameSpeedRequest {
    Pause,
    TogglePause,
    Unpause,
    SetSpeed(f32),
}

pub struct GameSpeedPlugin;

impl Plugin for GameSpeedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // app.init_resource::<GameSpeed>()
        //     .add_stage_before("update", "game_timer")
        //     .add_system_to_stage("game_timer", game_speed_update.system())
        //     .add_system_to_stage("game_timer", game_timer.system());
    }
}

fn game_speed_update(
    mut commands: Commands,
    mut game_time: ResMut<GameSpeed>,
    query: Query<(Entity, &GameSpeedRequest)>,
) {
    for (entity, game_speed) in query.iter() {
        match game_speed {
            GameSpeedRequest::Pause => game_time.pause(),
            GameSpeedRequest::Unpause => game_time.unpause(),
            GameSpeedRequest::SetSpeed(_speed) => unimplemented!(),
            GameSpeedRequest::TogglePause => game_time.toggle_pause(),
        }
        log::info!("Changing Game Speed: {:?}", game_speed);
        // game_time.game_speed = game_speed.new_game_speed;

        commands.despawn(entity);
    }
}

fn game_timer(time: Res<Time>, mut game_time: ResMut<GameSpeed>) {
    game_time.delta = time.delta_seconds() * game_time.game_speed;

    if game_time.delta < 0.01 {
        return;
    }

    game_time.elapsed_time += game_time.delta;
}
