use ai_player::{AIPlayerPlugin, Brain};
use bevy::prelude::*;
use doodl_jump::{DeathEvent, DoodlJumpPlugin, DoodlJumpSettings, ScrollHeight};

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f32 = 1. / GAME_FPS as f32;

pub fn trainer_plugin(app: &mut App) {
    app
        .add_observer(handle_death);
}

pub fn fitness(brain: &Brain) -> f32 {
    let game_plugin = DoodlJumpPlugin {
        settings: DoodlJumpSettings {
            fixed_dt: Some(GAME_DELTA_TIME),
            ..default()
        },
        ..default()
    };
    let ai_plugin = AIPlayerPlugin { 
        brain: brain.clone()
    };

    let mut fitness = 0.0;

    for _ in 0..NB_GAMES {
        let mut app = App::new();
        app.add_plugins((game_plugin.clone(), ai_plugin.clone()));
        fitness += eval_game_fitness(app);
    }

    fitness
}

pub fn handle_death(
    _event: On<DeathEvent>,
    mut commands: Commands,
) {
    commands.write_message(AppExit::Success);
}

pub fn eval_game_fitness(mut app: App) -> f32 {
    app.finish();
    loop {
        app.update();

        if app.should_exit().is_some() {
            return app.world().get_resource::<ScrollHeight>().unwrap().0;
        }
    }
}