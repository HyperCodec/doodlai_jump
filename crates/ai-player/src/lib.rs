use doodl_jump::{DoodlJumpSettings, Platform, Player, Velocity};
use neat::NeuralNetwork;
use bevy::prelude::*;

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f32 = 1. / GAME_FPS as f32;
pub const NB_GENERATIONS: usize = 1000;
pub const NB_GENOME_PER_GEN: usize = 2_500;
pub const MUTATION_RATE: f32 = 0.05;
pub const MUTATION_PASSES: usize = 3;

// 4 for player velxy + posxy, 1 for latest dt, 2 for each platform xy.
pub const AGENT_IN: usize = 15;

// left or right
pub const AGENT_OUT: usize = 2;

pub type Brain = NeuralNetwork<AGENT_IN, AGENT_OUT>;

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct AISource(pub Brain);

#[derive(Debug)]
pub struct AIPlayerPlugin {
    pub brain: Brain,
}

impl Plugin for AIPlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AISource(self.brain.clone()))
            .add_systems(PreUpdate, run_ai);
    }
}

pub fn run_ai(
    brain: Res<AISource>,
    player_q: Query<(&Transform, &Velocity), With<Player>>,
    platforms_q: Query<&Transform, With<Platform>>,
    time: Res<Time>,
    settings: Res<DoodlJumpSettings>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    let mut input = [0.0; 15];

    let (player_transform, player_velocity) = player_q.single().unwrap();
    input[0] = player_transform.translation.x;
    input[1] = player_transform.translation.y;
    input[2] = player_velocity.0.x;
    input[3] = player_velocity.0.y;

    let dt = settings.dt(&time);
    input[4] = dt;

    let mut i = 5;
    for platform_transform in platforms_q.iter() {
        let delta = platform_transform.translation - player_transform.translation;
        input[i] = delta.x;
        input[i+1] = delta.y;

        i += 2;
    }

    let output = brain.predict(input);
    if output[0] >= output[1] {
        keyboard.press(KeyCode::ArrowLeft);
    } else {
        keyboard.press(KeyCode::ArrowRight);
    }
}