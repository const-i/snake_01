
mod constants;
mod game;
mod render;
mod brain;

extern crate rayon;

use std::thread;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use crate::constants::*;
use crate::game::Game;
use crate::brain::{Layer, NN, Population};
use crate::render::Render;

fn main() {

    iterate_population(1000, 20, 50);
    //render_game_nn()
    
}


fn iterate_population(num_nn: u32, num_games: u32, num_generations: u32) {
    
    let mut pop = Game::get_population(num_nn);
    for i in 0..num_generations-1 {
        pop.fitness = population_play_parallel(&pop, num_games);
        let mut sorted_index = pop.get_sorted_index();
        println!("Gen: {}; Fitness: {}", i, pop.fitness[sorted_index[0]]);
        pop = pop.create_next_generation();
    }
    
    pop.fitness = population_play_parallel(&pop, num_games);
    let mut sorted_index = pop.get_sorted_index();
    println!("Final Fitness: {}", pop.fitness[sorted_index[0]]);
    let mut render = Render::new();
    render.run_nn(&pop.nn[sorted_index[0]]);
}


fn run_population(num_nn: u32, num_games: u32) -> Population {

    let mut pop = Game::get_population(num_nn);
    pop.fitness = population_play_parallel(&pop, num_games);
    pop

}

fn population_play_parallel(pop: &Population, num_games: u32) -> Vec<i64> {
    let fitness: Vec<i64> = pop.nn.par_iter()
                                  .map(|n| nn_play(n, num_games))
                                  .collect();
    fitness
}


fn population_play_concurrent(pop: Population, num_games: u32) -> Vec<i64> {
    let arc_pointer = Arc::new(Mutex::new(vec![]));
    let mut threads = vec![];
    for nn in pop.nn {
        threads.push(thread::spawn({
            let clone = Arc::clone(&arc_pointer);
            move || {
                let mut v = clone.lock().unwrap();
                let fit = nn_play(&nn, num_games);
                v.push(fit);
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    let lock = Arc::try_unwrap(arc_pointer).unwrap();
    let fitness = lock.into_inner().unwrap();
    fitness
}

fn nn_play(nn: &NN, num_games: u32) -> i64 {
    let mut game = Game::new();
    let mut fitness: i64 = 0;
    for _ in 0..num_games {
        game.init();
        let (score, time) = game.run_nn(nn);
        fitness += 500 * (score as i64) + 1 * (time as i64) - 2 * get_snake_dist(&game);
    }
    fitness
}

fn get_snake_dist(game: &Game) -> i64 {
    let dist_x = (game.snake.body[0].position.x as i64 - game.food.position.x as i64).abs();
    let dist_y = (game.snake.body[0].position.y as i64 - game.food.position.y as i64).abs();
    dist_x + dist_y
}


fn render_population_nn() {
    let pop = run_population(1000,10);
    let sorted_index = pop.get_sorted_index();
    println!("{:?}", pop.fitness[sorted_index[0]]);

    let mut render = Render::new();
    render.run_nn(&pop.nn[sorted_index[0]]);
}

fn render_game_nn() {

    let mut nn = Game::create_nn();
    let mut render = Render::new();
    render.run_nn(&mut nn);
}

fn render_game() {
    let mut render = Render::new();
    render.run();
}


