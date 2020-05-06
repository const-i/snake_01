pub static NAME: &str = "Snake v01";

// Board Dimensions
pub const BOARD_WIDTH: u8 = 10;
pub const BOARD_HEIGHT: u8 = 10;

// Colours
pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

// Neural Network Game
pub const NUM_INDIVIDUALS: u32 = 1000;
pub const NUM_GAMES_NN: u32 = 20;
pub const NUM_GENERATIONS: u32 = 20;
pub const NN_MAX_GAME_TIME: u32 = 100;

// Q-Learing Game
pub const NUM_GAMES_QL: u32 = 2000;
pub const NUM_QLS: u32 = 4; // Generally multiple of number of cores

// Genetic Algorithm Properties
pub const MUTATION_PROBABILITY: f64 = 0.005;
pub const CROSSOVER_PROBABILITY: f64 = 0.01;

// Game Render Properties
pub const BLOCK_SIZE: u32 = 20;
pub const RENDER_UPS: u64 = 20;
pub const RENDER_FPS_MAX: u64 = 20;

// Q-Learning Properties
pub const EPSILON_GREEDY: f64 = 0.01;
pub const LEARNING_RATE: f64 = 0.1;
pub const DISCOUNT_FACTOR: f64 = 0.9;
