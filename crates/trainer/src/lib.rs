pub mod game_interop;
use neat::NeuralNetwork;

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f64 = 1. / GAME_FPS as f64;
pub const NB_GENERATIONS: usize = 1000;
pub const NB_GENOME_PER_GEN: usize = 2_500;
pub const MUTATION_RATE: f32 = 0.05;
pub const MUTATION_PASSES: usize = 3;

// 4 for player velxy + posxy, 2 for each platform xy, 1 for latest dt.
pub const AGENT_IN: usize = 15;

// left or right
pub const AGENT_OUT: usize = 2;

pub type Brain = NeuralNetwork<AGENT_IN, AGENT_OUT>;
