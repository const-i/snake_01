mod constants;
mod game;
mod gen_alg;
mod pathfind;
mod qlearn;
mod render;

extern crate rayon;

use rayon::prelude::*;

use crate::constants::*;
use crate::game::{Brain, Game};
use crate::gen_alg::{Population, NN};
use crate::qlearn::QLearner;
use crate::render::Render;

enum GameType {
    Human,
    GeneticAlgorithm,
    QLearning,
    AStar,
    Hamiltonian,
}

fn main() {
    let game_type = GameType::AStar;

    match game_type {
        GameType::Human => render_game(),
        GameType::GeneticAlgorithm => {
            iterate_population(NUM_INDIVIDUALS, NUM_GAMES_NN, NUM_GENERATIONS, fitness_function_nn)
        }
        GameType::QLearning => iterate_qls(NUM_QLS, NUM_GAMES_QL, fitness_function_ql),
        GameType::AStar => render_astar_game(),
        GameType::Hamiltonian => render_hamiltonian_game(),
    }
}

fn fitness_function_nn(_delta_t: i64, dist_before: i64, dist_after: i64, snake_eat: i64, snake_dead: i64) -> f64 {
    let mut fitness: f64 = 0.0_f64;
    // Distance
    if dist_after < dist_before {
        fitness += 0.3_f64;
    } else {
        fitness -= 0.5_f64;
    }
    if dist_before >= 2 && dist_after < 2 {
        fitness += 0.5_f64;
    }
    // Time
    fitness += 0.1_f64;
    // Food
    if snake_eat > 0 {
        fitness += 5.0_f64;
    }
    // Dead
    if snake_dead > 0 {
        fitness -= 1.0_f64;
    }
    fitness
}

fn fitness_function_ql(_delta_t: i64, dist_before: i64, dist_after: i64, snake_eat: i64, snake_dead: i64) -> f64 {
    let mut fitness: f64 = 0.0_f64;
    // Distance
    if dist_after < dist_before {
        fitness += 0.3_f64;
    } else {
        fitness -= 0.7_f64;
    }
    if dist_before >= 2 && dist_after < 2 {
        fitness += 0.5_f64;
    }
    // Time
    fitness += 0.1_f64;
    // Food
    if snake_eat > 0 {
        fitness += 5.0_f64;
    }
    // Dead
    if snake_dead > 0 {
        fitness -= 1.0_f64;
    }
    fitness
}

// --------------------------------------------------------------------------------------
// ----------------------------------Neural Network--------------------------------------
// --------------------------------------------------------------------------------------

fn iterate_population(
    num_nn: u32,
    num_games: u32,
    num_generations: u32,
    fitness_function: fn(i64, i64, i64, i64, i64) -> f64,
) {
    let mut pop = Population::new_defined(num_nn, &[[8, 8], [8, 4]]);
    for i in 0..num_generations - 1 {
        pop.fitness = population_play_parallel(&mut pop.nn, num_games, fitness_function);
        let sorted_index = pop.get_sorted_index();
        println!("Gen: {}; Fitness: {}", i, pop.fitness[sorted_index[0]]);
        pop = pop.create_next_generation();
    }

    pop.fitness = population_play_parallel(&mut pop.nn, num_games, fitness_function);
    let sorted_index = pop.get_sorted_index();
    println!("Final Fitness: {}", pop.fitness[sorted_index[0]]);
    let mut render = Render::new();
    render.run_brain(&mut pop.nn[sorted_index[0]]);
}

fn population_play_parallel(
    nns: &mut Vec<NN>,
    num_games: u32,
    fitness_function: fn(i64, i64, i64, i64, i64) -> f64,
) -> Vec<f64> {
    let fitness: Vec<f64> = nns
        .par_iter_mut()
        .map(|n| play_brain(n, num_games, fitness_function))
        .collect();
    fitness
}

// --------------------------------------------------------------------------------------
// ----------------------------------Q Learning------------------------------------------
// --------------------------------------------------------------------------------------

fn iterate_qls(num_qls: u32, num_games: u32, fitness_function: fn(i64, i64, i64, i64, i64) -> f64) {
    let mut qls: Vec<QLearner> = (0..num_qls).map(|_| QLearner::new(8, 4)).collect();
    let max_i = ql_play_parallel(&mut qls, num_games, fitness_function);
    let mut render = Render::new();
    render.run_brain(&mut qls[max_i]);
}

fn ql_play_parallel(
    qls: &mut Vec<QLearner>,
    num_games: u32,
    fitness_function: fn(i64, i64, i64, i64, i64) -> f64,
) -> usize {
    let fitness: Vec<f64> = qls
        .par_iter_mut()
        .map(|ql| play_brain(ql, num_games, fitness_function))
        .collect();
    fitness
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

// --------------------------------------------------------------------------------------
// ----------------------------------Generic Brain---------------------------------------
// --------------------------------------------------------------------------------------

fn play_brain<T: Brain>(brain: &mut T, num_games: u32, fitness_function: fn(i64, i64, i64, i64, i64) -> f64) -> f64 {
    let mut game = Game::new();
    let mut fitness: f64 = 0f64;
    for _ in 0..num_games {
        game.init();
        fitness += game.run_brain(brain, fitness_function);
    }
    fitness / num_games as f64
}

// --------------------------------------------------------------------------------------
// ----------------------------------AStar Game------------------------------------------
// --------------------------------------------------------------------------------------

fn render_astar_game() {
    let mut render = Render::new();
    render.run_astar();
}

// --------------------------------------------------------------------------------------
// ----------------------------------Hamiltonian Game------------------------------------
// --------------------------------------------------------------------------------------

fn render_hamiltonian_game() {
    let mut render = Render::new();
    render.run_hamiltonian();
}

// --------------------------------------------------------------------------------------
// ----------------------------------Human Game------------------------------------------
// --------------------------------------------------------------------------------------

fn render_game() {
    let mut render = Render::new();
    render.run();
}
