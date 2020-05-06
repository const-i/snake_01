extern crate rand;

use rand::Rng;
use std::collections::VecDeque;
use std::fmt;

use crate::brain::{Layer, Population, NN};
use crate::constants::*;
use crate::qlearn::QLearner;

#[derive(Copy, Clone)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    fn new() -> Position {
        Position {
            x: BOARD_WIDTH / 2,
            y: BOARD_HEIGHT / 2,
        }
    }

    fn new_offset(x: i8, y: i8) -> Position {
        let mut pos = Position::new();
        pos.offset(x, y);
        pos
    }

    fn offset(&mut self, x: i8, y: i8) {
        self.x = Position::calc_offset(self.x, x, BOARD_WIDTH);
        self.y = Position::calc_offset(self.y, y, BOARD_HEIGHT);
    }

    fn calc_offset(val: u8, offset: i8, max_val: u8) -> u8 {
        if (val == 0 && offset < 0) || (val >= max_val - 1 && offset > 0) {
            val
        } else {
            let off_max = offset as i16 % max_val as i16;
            if off_max < 0 {
                let x1 = off_max as u8;
                let x2 = x1 - std::u8::MAX / 2 - 1 + max_val;
                let x3 = x2 - std::u8::MAX / 2 - 1;
                (val + x3) % max_val
            } else {
                (val + off_max as u8) % max_val
            }
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point").field("x", &self.x).field("y", &self.y).finish()
    }
}

pub struct Block {
    pub position: Position,
    pub colour: [f32; 4],
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    fn opposite(&mut self) -> Direction {
        match self {
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
        }
    }
}

pub struct Snake {
    pub body: VecDeque<Block>,
    pub direction: Direction,
    pub alive: bool,
    pub eat: bool,
}

impl Snake {
    fn new() -> Snake {
        Snake {
            body: VecDeque::from(vec![
                Block {
                    position: Position::new(),
                    colour: YELLOW,
                },
                Block {
                    position: Position::new_offset(-1, 0),
                    colour: GREEN,
                },
                Block {
                    position: Position::new_offset(-2, 0),
                    colour: GREEN,
                },
            ]),
            direction: Direction::RIGHT,
            alive: true,
            eat: false,
        }
    }

    fn update(&mut self, mut dir: Direction) {
        if self.direction == dir.opposite() {
            // Do nothing
        } else {
            self.direction = dir;
        }
    }

    fn perform_next(&mut self, food_pos: &mut Position) {
        if self.alive {
            let next_pos = self.next_head_pos();
            if self.check_collide_wall(next_pos) || self.check_collide_body(next_pos) {
                self.alive = false;
            } else if self.check_eat_food(next_pos, *food_pos) {
                self.eat_next(food_pos);
                self.eat = true;
            } else {
                self.move_next();
            }
        }
    }

    fn next_head_pos(&mut self) -> Position {
        let mut current_head = self.body[0].position;
        match self.direction {
            Direction::RIGHT => current_head.offset(1, 0),
            Direction::UP => current_head.offset(0, -1),
            Direction::LEFT => current_head.offset(-1, 0),
            Direction::DOWN => current_head.offset(0, 1),
        }
        current_head
    }

    fn check_collide_wall(&self, next_pos: Position) -> bool {
        self.body[0].position == next_pos
    }

    fn check_collide_body(&self, pos: Position) -> bool {
        self.body.iter().any(|block| block.position == pos)
    }

    fn check_eat_food(&self, next_pos: Position, food_pos: Position) -> bool {
        next_pos == food_pos
    }

    fn move_next(&mut self) {
        for i in (1..self.body.len()).rev() {
            self.body[i].position = self.body[i - 1].position;
        }
        self.body[0].position = self.next_head_pos();
    }

    fn eat_next(&mut self, pos: &mut Position) {
        let head = Block {
            position: *pos,
            colour: YELLOW,
        };
        self.body.push_front(head);
        self.body[1].colour = GREEN;
    }
}

pub struct Game {
    pub snake: Snake,
    pub food: Block,
    pub time: u32,
    pub score: u32,
}

impl Game {
    pub fn new() -> Game {
        Game {
            snake: Snake::new(),
            food: Block {
                position: Position::new(),
                colour: RED,
            },
            time: 0,
            score: 0,
        }
    }

    pub fn init(&mut self) {
        self.snake = Snake::new();
        self.food.position = self.get_food_pos();
        self.time = 0;
        self.score = 0;
    }

    pub fn update(&mut self, dir: Direction) {
        self.snake.update(dir);
    }

    pub fn next_tick(&mut self, _dt: f64) {
        if self.snake.alive {
            self.snake.perform_next(&mut self.food.position);
            self.time += 1;
            if self.snake.eat {
                self.score += 1;
                self.food.position = self.get_food_pos();
                self.snake.eat = false;
            }
        }
    }

    pub fn create_nn() -> NN {
        let mut nn = NN::new();
        let layer1 = Layer::new(8, 8).unwrap();
        let layer2 = Layer::new(8, 4).unwrap();
        nn.add(layer1);
        nn.add(layer2);
        nn
    }

    pub fn run_nn(&mut self, nn: &NN, fitness_function: fn(i64, i64, i64, i64, i64) -> f64) -> f64 {
        self.init();
        let mut fitness: f64 = 0f64;
        while self.snake.alive {
            let dir = self.get_dir_nn(&nn);
            self.update(dir);

            // Before moving store some results
            let dist_before = self.get_food_dist();
            let time_before = self.time;

            // Make the move
            self.next_tick(1f64);

            // After the move store some results
            let dist_after = self.get_food_dist();
            let time_after = self.time;
            let snake_eat = if self.snake.eat { 1i64 } else { 0i64 };
            let snake_dead = if self.snake.alive { 0i64 } else { 1i64 };

            // Add to fitness
            fitness += fitness_function(
                time_after as i64 - time_before as i64,
                dist_before,
                dist_after,
                snake_eat,
                snake_dead,
            );

            if self.time >= NN_MAX_GAME_TIME {
                self.snake.alive = false;
            }
        }
        fitness
    }

    pub fn run_ql(&mut self, ql: &mut QLearner, fitness_function: fn(i64, i64, i64, i64, i64) -> f64) -> f64 {
        self.init();
        let mut fitness: f64 = 0f64;
        while self.snake.alive {
            let state_initial = self.get_nn_inputs();
            let action = ql.get_action(&state_initial);
            let dir = self.get_direction_from_index(action);
            self.update(dir);

            // Before moving store some results
            let dist_before = self.get_food_dist();
            let time_before = self.time;

            // Make the move
            self.next_tick(1f64);

            // After the move store some results
            let dist_after = self.get_food_dist();
            let time_after = self.time;
            let snake_eat = if self.snake.eat { 1i64 } else { 0i64 };
            let snake_dead = if self.snake.alive { 0i64 } else { 1i64 };

            // Get fitness & State
            let fit = fitness_function(
                time_after as i64 - time_before as i64,
                dist_before,
                dist_after,
                snake_eat,
                snake_dead,
            );
            let state_final = self.get_nn_inputs();
            fitness += fit;

            // Update the Q Matrix
            ql.update_q(&state_initial, action, fit, &state_final);

            // End if we are out of time
            if self.time >= NN_MAX_GAME_TIME {
                self.snake.alive = false;
            }
        }
        fitness
    }

    pub fn get_dir_nn(&mut self, nn: &NN) -> Direction {
        let nn_out = nn.propagate(self.get_nn_inputs()).unwrap();
        let i_max = nn_out
            .iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .max_by(|a, b| a.partial_cmp(b).expect("Nan!"))
            .unwrap()
            .1;
        self.get_direction_from_index(i_max)
    }

    pub fn get_dir_ql(&mut self, ql: &mut QLearner) -> Direction {
        self.get_direction_from_index(ql.get_action(&self.get_nn_inputs()))
    }

    pub fn get_direction_from_index(&self, index: usize) -> Direction {
        match index {
            0 => Direction::RIGHT,
            1 => Direction::UP,
            2 => Direction::LEFT,
            3 => Direction::DOWN,
            _ => self.snake.direction,
        }
    }

    pub fn get_population(num_nn: u32) -> Population {
        let mut pop = Population::new();
        for _ in 0..num_nn {
            pop.add(Game::create_nn());
        }
        pop
    }

    fn get_food_pos(&mut self) -> Position {
        let mut rng = rand::thread_rng();
        loop {
            let pos = Position {
                x: rng.gen_range(0, BOARD_WIDTH),
                y: rng.gen_range(0, BOARD_HEIGHT),
            };
            if !self.snake.check_collide_body(pos) {
                return pos;
            }
        }
    }

    pub fn get_food_dist(&self) -> i64 {
        let dist_x = (self.snake.body[0].position.x as i64 - self.food.position.x as i64).abs();
        let dist_y = (self.snake.body[0].position.y as i64 - self.food.position.y as i64).abs();
        dist_x + dist_y
    }

    pub fn get_nn_inputs(&self) -> Vec<f64> {
        let head_pos = self.snake.body[0].position;
        let food_pos = self.food.position;

        let mut pos_right = head_pos;
        pos_right.offset(1, 0);
        let right_dead = self.get_pos_dead(pos_right);
        let right_food = if food_pos.y == head_pos.y && food_pos.x > head_pos.x {
            1f64
        } else {
            0f64
        };

        let mut pos_up = head_pos;
        pos_up.offset(0, -1);
        let up_dead = self.get_pos_dead(pos_up);
        let up_food = if food_pos.x == head_pos.x && food_pos.y > head_pos.y {
            1f64
        } else {
            0f64
        };

        let mut pos_left = head_pos;
        pos_left.offset(-1, 0);
        let left_dead = self.get_pos_dead(pos_left);
        let left_food = if food_pos.y == head_pos.y && food_pos.x < head_pos.x {
            1f64
        } else {
            0f64
        };

        let mut pos_down = head_pos;
        pos_down.offset(0, 1);
        let down_dead = self.get_pos_dead(pos_down);
        let down_food = if food_pos.x == head_pos.x && food_pos.y < head_pos.y {
            1f64
        } else {
            0f64
        };

        vec![
            right_dead, right_food, up_dead, up_food, left_dead, left_food, down_dead, down_food,
        ]
    }

    fn get_pos_dead(&self, pos: Position) -> f64 {
        if self.snake.check_collide_wall(pos) || self.snake.check_collide_body(pos) {
            1f64
        } else {
            0f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new();
        assert_eq!(pos.x, BOARD_WIDTH / 2);
        assert_eq!(pos.y, BOARD_HEIGHT / 2);
    }

    #[test]
    fn test_position_offset() {
        let pos1 = Position::new();
        let mut pos2 = Position::new();
        pos2.offset(0, 0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset(1, 0);
        assert_eq!(pos2.x, pos1.x + 1);
        pos2 = Position::new();
        pos2.offset(-1, 0);
        assert_eq!(pos2.x + 1, pos1.x);
        pos2 = Position::new();
        pos2.offset(BOARD_WIDTH as i8, 0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset((BOARD_WIDTH as i8) + 1, 0);
        assert_eq!(pos2.x, pos1.x + 1);
        pos2 = Position::new();
        pos2.offset(-(BOARD_WIDTH as i8), 0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset(-(BOARD_WIDTH as i8) - 1, 0);
        assert_eq!(pos2.x + 1, pos1.x);
    }

    #[test]
    fn test_position_new_offset() {
        let mut pos1 = Position::new();
        let mut pos2 = Position::new_offset(0, 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(1, 0);
        pos2 = Position::new_offset(1, 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-1, 0);
        pos2 = Position::new_offset(-1, 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(BOARD_WIDTH as i8, 0);
        pos2 = Position::new_offset(BOARD_WIDTH as i8, 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(BOARD_WIDTH as i8 + 1, 0);
        pos2 = Position::new_offset(BOARD_WIDTH as i8 + 1, 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-(BOARD_WIDTH as i8), 0);
        pos2 = Position::new_offset(-(BOARD_WIDTH as i8), 0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-(BOARD_WIDTH as i8) - 1, 0);
        pos2 = Position::new_offset(-(BOARD_WIDTH as i8) - 1, 0);
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_snake_new() {
        let snake = Snake::new();
        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.direction, Direction::RIGHT);
        let pos1 = Position::new();
        let pos2 = Position::new_offset(-1, 0);
        let pos3 = Position::new_offset(-2, 0);
        assert_eq!(snake.body[0].position, pos1);
        assert_eq!(snake.body[1].position, pos2);
        assert_eq!(snake.body[2].position, pos3);
    }

    #[test]
    fn test_snake_next_head_pos() {
        let mut snake = Snake::new();
        let next_pos = snake.next_head_pos();
        let pos = Position::new_offset(1, 0);
        assert_eq!(next_pos, pos);
    }

    #[test]
    fn test_snake_check_collide_wall() {
        let snake = Snake::new();
        let mut pos = Position::new();
        assert!(snake.check_collide_wall(pos));
        pos = Position::new_offset(1, 0);
        assert!(!snake.check_collide_wall(pos));
    }

    #[test]
    fn test_snake_check_collide_body() {
        let snake = Snake::new();
        let mut pos = Position::new();
        assert!(snake.check_collide_body(pos));
        pos.offset(1, 0);
        assert!(!snake.check_collide_body(pos));
        pos.offset(-3, 0);
        assert!(snake.check_collide_body(pos));
        pos.offset(-4, 0);
        assert!(!snake.check_collide_body(pos));
    }

    #[test]
    fn test_snake_move_next() {
        let mut snake = Snake::new();
        snake.move_next();
        let mut pos = Position::new_offset(1, 0);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-1, 0);
        assert_eq!(snake.body[2].position, pos);
        pos = Position::new_offset(-2, 0);
        assert!(!snake.check_collide_body(pos));
    }

    #[test]
    fn test_snake_eat_next() {
        let mut snake = Snake::new();
        let mut next_pos = snake.next_head_pos();
        snake.eat_next(&mut next_pos);
        assert_eq!(snake.body.len(), 4);
        let mut pos = Position::new_offset(1, 0);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-2, 0);
        assert_eq!(snake.body[3].position, pos);
        pos = Position::new_offset(-3, 0);
        assert!(!snake.check_collide_body(pos));
    }

    #[test]
    fn test_snake_update() {
        let mut snake = Snake::new();
        snake.update(Direction::LEFT);
        assert_eq!(snake.direction, Direction::RIGHT);
        snake = Snake::new();
        snake.update(Direction::UP);
        assert_eq!(snake.direction, Direction::UP);
        snake = Snake::new();
        snake.update(Direction::DOWN);
        assert_eq!(snake.direction, Direction::DOWN);
        snake = Snake::new();
        snake.update(Direction::RIGHT);
        assert_eq!(snake.direction, Direction::RIGHT);
    }

    #[test]
    fn test_snake_perform_next() {
        let mut snake = Snake::new();
        let mut food = Position::new_offset(1, 0);
        snake.perform_next(&mut food);
        assert_eq!(snake.body.len(), 4);
        food = Position::new_offset(0, 1);
        snake.update(Direction::UP);
        snake.perform_next(&mut food);
        let mut pos = Position::new_offset(1, -1);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-1, 0);
        assert_eq!(snake.body[3].position, pos);
        pos = Position::new_offset(-2, 0);
        assert!(!snake.check_collide_body(pos));
        // Check whether we collide with the walls and die
        while snake.body[0].position.y >= 1 {
            snake.perform_next(&mut food);
        }
        let next_pos = snake.next_head_pos();
        assert!(snake.check_collide_wall(next_pos));
        assert!(snake.alive);
        snake.perform_next(&mut food);
        assert!(!snake.alive);
        // Check whether we collide with ourself and die
        snake = Snake::new();
        food = Position::new_offset(1, 0);
        snake.perform_next(&mut food);
        food = Position::new_offset(2, 0);
        snake.perform_next(&mut food);
        assert_eq!(snake.body.len(), 5);
        snake.update(Direction::UP);
        snake.perform_next(&mut food);
        snake.update(Direction::LEFT);
        snake.perform_next(&mut food);
        snake.update(Direction::DOWN);
        snake.perform_next(&mut food);
        assert!(!snake.alive);
        // Check whether we collide with the walls and die
        snake = Snake::new();
        snake.update(Direction::DOWN);
        snake.perform_next(&mut food);
        pos = Position::new_offset(0, 1);
        assert_eq!(snake.body[0].position, pos);
        while snake.body[0].position.y <= BOARD_HEIGHT - 2 {
            snake.perform_next(&mut food);
        }
        let next_pos = snake.next_head_pos();
        println!(
            "Head: {:?}; Next: {:?}; Alive: {}",
            snake.body[0].position, next_pos, snake.alive
        );
        snake.perform_next(&mut food);
        println!(
            "Head: {:?}; Next: {:?}; Alive: {}",
            snake.body[0].position, next_pos, snake.alive
        );
        assert!(!snake.alive);
    }

    #[test]
    fn test_game_new() {
        let game = Game::new();
        assert_eq!(game.snake.body[0].position, game.food.position);
        assert_eq!(game.snake.body.len(), 3);
        assert_eq!(game.time, 0);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn test_game_get_food_pos() {
        let mut game = Game::new();
        for _ in 0..10 {
            let pos = game.get_food_pos();
            assert!(!game.snake.check_collide_body(pos));
        }
    }

    #[test]
    fn test_game_init() {
        let mut game = Game::new();
        game.init();
        assert!(!game.snake.check_collide_body(game.food.position));
        assert_eq!(game.time, 0);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn test_game_update() {
        let mut game = Game::new();
        game.init();
        assert_eq!(game.snake.direction, Direction::RIGHT);
        game.update(Direction::UP);
        assert_eq!(game.snake.direction, Direction::UP);
        game.update(Direction::DOWN);
        assert_eq!(game.snake.direction, Direction::UP);
    }

    #[test]
    fn test_game_next_tick() {
        let mut game = Game::new();
        game.init();
        let mut pos = Position::new();
        assert_eq!(game.snake.body[0].position, pos);
        game.next_tick(0.1);
        pos.offset(1, 0);
        assert_eq!(game.snake.body[0].position, pos);
    }

    #[test]
    fn test_game_get_dir_nn() {
        let mut game = Game::new();
        game.init();
        let mut nn = NN::new();
        let layer1 = Layer::new(8, 8).unwrap();
        let layer2 = Layer::new(8, 4).unwrap();
        nn.add(layer1);
        nn.add(layer2);
        let board = game.get_nn_inputs();
        println!("{:?}", board);
        let out = nn.propagate(board);
        println!("{:?}", out);
        let dir = game.get_dir_nn(&mut nn);
        println!("{:?}", dir);
        //assert!(false);
    }

    #[test]
    fn test_game_create_nn() {
        let nn = Game::create_nn();
        assert_eq!(nn.layers.len(), 2);
        assert_eq!(nn.layers[0].num_inputs, 8);
        assert_eq!(nn.layers[1].num_neurons, 4);
    }

    #[test]
    fn test_game_run_nn() {
        use std::cmp;

        fn fitness_function(_delta_t: i64, dist_before: i64, dist_after: i64, snake_eat: i64, _snake_dead: i64) -> f64 {
            let mut fitness: f64 = 0.0_f64;
            if dist_after < dist_before {
                fitness += 1.0_f64;
            } else {
                fitness -= 2.0_f64;
            }
            fitness += 1.0_f64; // Time
            fitness += 100.0_f64 * snake_eat as f64;
            fitness
            //500 * score + time - 2 * food_distance
            //100 * score + score * 1000 / (time + 1)  + time - food_distance
        }

        let mut game = Game::new();
        game.init();
        let mut nn = NN::new();
        let layer1 = Layer::new(8, 8).unwrap();
        let layer2 = Layer::new(8, 4).unwrap();
        nn.add(layer1);
        nn.add(layer2);
        game.run_nn(&mut nn, fitness_function);
        println!("{}", game.time);
        assert!(game.time >= cmp::min(BOARD_WIDTH as u32, BOARD_HEIGHT as u32) / 2);
    }

    #[test]
    fn test_game_get_nn_inputs() {
        let mut game = Game::new();
        game.init();
        println!("****Start****");
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs.len(), 8);
        println!("****Right Food****");
        game.food.position.x = game.snake.body[0].position.x + 1;
        game.food.position.y = game.snake.body[0].position.y;
        inputs = game.get_nn_inputs();
        assert_eq!(inputs[1], 1f64);
        println!("****Right Dead****");
        for _ in 0..BOARD_WIDTH / 2 - 1 {
            game.next_tick(0.1);
        }
        let inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        println!("****Up Right Dead****");
        game.update(Direction::UP);
        for _ in 0..BOARD_HEIGHT / 2 {
            game.next_tick(0.1);
        }
        let inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[2], 1f64);
        println!("****Up Right Dead****");
        game.update(Direction::LEFT);
        for _ in 0..BOARD_WIDTH - 1 {
            game.next_tick(0.1);
        }
        let inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[2], 1f64);
        assert_eq!(inputs[4], 1f64);
        println!("****Down Left Dead****");
        game.update(Direction::DOWN);
        for _ in 0..BOARD_HEIGHT - 1 {
            game.next_tick(0.1);
        }
        let inputs = game.get_nn_inputs();
        assert_eq!(inputs[2], 1f64);
        assert_eq!(inputs[4], 1f64);
        assert_eq!(inputs[6], 1f64);
        println!("****Down Right Dead****");
        game.update(Direction::RIGHT);
        for _ in 0..BOARD_WIDTH - 1 {
            game.next_tick(0.1);
        }
        let inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[4], 1f64);
        assert_eq!(inputs[6], 1f64);
    }
}
