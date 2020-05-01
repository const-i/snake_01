
extern crate rand;

use std::fmt;
use std::thread;
use std::collections::VecDeque;
use rand::Rng;

use crate::constants::*;
use crate::brain::{NN, Layer, Population};

const NN_MAX_TIME: u32 = 200;


struct Board {
    width: u8,
    height: u8,
    board: Vec<Vec<f64>>,
}

impl Board {

    fn new(width: u8, height: u8) -> Board {
        Board {
            width: width,
            height: height,
            board: vec![vec![0.5f64; usize::from(width)]; usize::from(height)],
        }
    }
}


#[derive(Copy, Clone)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    
    fn new() -> Position {
        Position {
            x: BOARD_WIDTH/2,
            y: BOARD_HEIGHT/2,
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
        if val == 0 && offset < 0 {
            val
        }
        else if val >= max_val - 1 &&  offset > 0 {
            val
        }
        else {
            let off_max = offset as i16 % max_val as i16;
            if off_max < 0 {
                let x1 = off_max as u8;
                let x2 = x1 - std::u8::MAX/2 - 1 + max_val;
                let x3 = x2 - std::u8::MAX/2 - 1;
                (val + x3) % max_val
            }
            else {
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
        f.debug_struct("Point")
         .field("x", &self.x)
         .field("y", &self.y)
         .finish()
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
            Direction::UP    => Direction::DOWN,
            Direction::DOWN  => Direction::UP,
            Direction::LEFT  => Direction::RIGHT,
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
                Block {position: Position::new(), colour: YELLOW},
                Block {position: Position::new_offset(-1,0), colour: GREEN},
                Block {position: Position::new_offset(-2,0), colour: GREEN},
            ]),
            direction: Direction::RIGHT,
            alive: true,
            eat: false,
        }
    }
    
    fn update(&mut self, mut dir: Direction) {
        if self.direction == dir.opposite() {
            // Do nothing
        }
        else {
            self.direction = dir;
        }
    }
    
    fn perform_next(&mut self, food_pos: &mut Position) {
        // Calculate next head position
        // Check if we collide: Die if we do and exit
        // Check it we eat: Grow if we do, and create new fruit
        // IF nothing else, then move
        if self.alive {
            let next_pos = self.next_head_pos();
            if self.check_collide_wall(&next_pos) || self.check_collide_body(&next_pos) {
                self.alive = false;
            }
            else if self.check_eat_food(next_pos, *food_pos) {
                self.eat_next(food_pos);
                self.eat = true;
            }
            else {
                self.move_next();
            }
        }
    }
    
    fn next_head_pos(&mut self) -> Position {
        let mut current_head = self.body[0].position.clone();
        match self.direction {
            Direction::RIGHT => current_head.offset(1,0),
            Direction::UP    => current_head.offset(0,-1),
            Direction::LEFT  => current_head.offset(-1,0),
            Direction::DOWN  => current_head.offset(0,1),
        }
        current_head
    }
    
    fn check_collide_wall(&self, next_pos: &Position) -> bool {
        self.body[0].position == *next_pos
        //pos.x <= 0 || pos.y <= 0 || pos.x >= BOARD_WIDTH - 1 || pos.y >= BOARD_HEIGHT - 1
    }
    
    fn check_collide_body(&self, pos: &Position) -> bool {
        self.body.iter().any(|block| block.position == *pos)
    }
    
    fn check_eat_food(&self, next_pos: Position, food_pos: Position) -> bool {
        next_pos == food_pos
    }
    
    fn move_next(&mut self) {
        for i in (1..self.body.len()).rev() {
            self.body[i].position = self.body[i-1].position;
        }
        self.body[0].position = self.next_head_pos();
    }
    
    fn eat_next(&mut self, pos: &mut Position) {
        let head = Block {position: *pos, colour: YELLOW};
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
            food: Block {position: Position::new(), colour: RED},
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
    
    pub fn next_tick(&mut self, dt: f64) {
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
    
    fn run(&mut self) {
        
        while self.snake.alive {
            self.snake.perform_next(&mut self.food.position);
            self.time += 1;
            if self.snake.eat { 
                self.score += 1;
                self.food.position = self.get_food_pos();
            }
            assert!(self.time < 20);
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
    
    
    pub fn run_nn(&mut self, nn: &NN) -> (u32, u32) {
        self.init();
        while self.snake.alive {
            let dir = self.get_dir_nn(&nn);
            self.update(dir);
            self.next_tick(1f64);
            if self.time >= NN_MAX_TIME {
                self.snake.alive = false;
            }
            //assert!(self.time < 20);
        }
        (self.score, self.time)
    }
    
    /*
    pub fn get_nn_input(&mut self) -> Vec<f64> {
        let board = self.get_board().board.clone();
        board.into_iter()
             .flatten()
             .collect::<Vec<f64>>()
    }
    */
    
    pub fn get_dir_nn(&mut self, nn: &NN) -> Direction {
        let nn_out = nn.propagate(self.get_nn_inputs()).unwrap();
        let i_max = nn_out.iter()
                          .enumerate()
                          .map(|(i,v)| (v,i))
                          .max_by(|a,b| a.partial_cmp(b).expect("Nan!"))
                          .unwrap()
                          .1;
        match i_max {
            0 => Direction::RIGHT,
            1 => Direction::UP,
            2 => Direction::LEFT,
            3 => Direction::DOWN,
            _ => self.snake.direction,
        }
    }
    

    pub fn get_population(num_nn: u32) -> Population {
        let mut pop = Population::new();
        for i in 0..num_nn {
            pop.add(Game::create_nn());
        }
        pop
    }
    
    /*
    pub fn population_play(&mut self, pop: &mut Population, num_games: u32) {
        for i in 0..pop.nn.len() {
            pop.nn[i].fitness = 0;
            for _ in 0..num_games {
                self.init();
                self.run_nn(&mut pop.nn[i]);
                pop.nn[i].fitness += self.time * 1 + self.score * 10;
            }
        }
    }
    */

    fn get_food_pos(&mut self) -> Position {
        let mut rng = rand::thread_rng();
        loop {
            let mut pos = Position{x: rng.gen_range(0, BOARD_WIDTH), y: rng.gen_range(0, BOARD_HEIGHT)};
            if !self.snake.check_collide_body(&pos) {
                return pos;
            }
        }
    }
    
    fn get_board(&mut self) -> Board {
    
        let mut board = Board::new(BOARD_WIDTH, BOARD_HEIGHT);
        board.board[self.food.position.y as usize][self.food.position.x as usize] = 1f64;
        for i in 0..self.snake.body.len() {
            board.board[self.snake.body[i].position.y as usize][self.snake.body[i].position.x as usize] = 0f64;
        }
        board
    }
    
    fn get_nn_inputs(&self) -> Vec<f64> {
        let head_pos = self.snake.body[0].position;
        let food_pos = self.food.position;
        //println!("Head: {}, {}", head_pos.x, head_pos.y);
        //println!("Food: {}, {}", food_pos.x, food_pos.y);
        
        let mut pos_right = head_pos.clone();
        pos_right.offset(1,0);
        //println!("Right: {}, {}", pos_right.x, pos_right.y);
        let right_dead = self.get_pos_dead(&pos_right);
        let right_food = if food_pos.y == head_pos.y && food_pos.x > head_pos.x { 1f64 }
                         else                                                   { 0f64 };
        //println!("Dead: {}; Food: {}", right_dead, right_food);

        let mut pos_up = head_pos.clone();
        pos_up.offset(0,-1);
        //println!("Up: {}, {}", pos_up.x, pos_up.y);
        let up_dead = self.get_pos_dead(&pos_up);
        let up_food = if food_pos.x == head_pos.x && food_pos.y > head_pos.y { 1f64 }
                      else                                                   { 0f64 };
        //println!("Dead: {}; Food: {}", up_dead, up_food);
        
        let mut pos_left = head_pos.clone();
        pos_left.offset(-1,0);
        //println!("Left: {}, {}", pos_left.x, pos_left.y);
        let left_dead = self.get_pos_dead(&pos_left);
        let left_food = if food_pos.y == head_pos.y && food_pos.x < head_pos.x { 1f64 }
                        else                                                   { 0f64 };
        //println!("Dead: {}; Food: {}", left_dead, left_food);
        
        let mut pos_down = head_pos.clone();
        pos_down.offset(0,1);
        //println!("Down: {}, {}", pos_down.x, pos_down.y);
        let down_dead =  self.get_pos_dead(&pos_down);
        let down_food = if food_pos.x == head_pos.x && food_pos.y < head_pos.y { 1f64 }
                        else                                                   { 0f64 };
        //println!("Dead: {}; Food: {}", down_dead, down_food);
        
        
        vec![right_dead, right_food,
             up_dead, up_food,
             left_dead, left_food,
             down_dead, down_food,
        ]
    }
    
    fn get_pos_dead(&self, pos: &Position) -> f64 {
        let pos_dead = if self.snake.check_collide_wall(&pos) ||
                           self.snake.check_collide_body(&pos) {
                          1f64
                       }
                       else {
                          0f64
                       };
        pos_dead
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new() {
        let board = Board::new(2,3);
        assert_eq!(board.width, 2);
        assert_eq!(board.height, 3);
        assert_eq!(board.board.len(), usize::from(board.height));
        assert_eq!(board.board[0].len(), usize::from(board.width));
        assert_eq!(board.board[0][0], 0.5f64);
        assert_eq!(board.board[usize::from(board.height)-1][usize::from(board.width)-1], 0.5f64);
    }
    
    #[test]
    fn test_position_new() {
        let pos = Position::new();
        assert_eq!(pos.x, BOARD_WIDTH/2);
        assert_eq!(pos.y, BOARD_HEIGHT/2);
    }
    
    #[test]
    fn test_position_offset() {
        let pos1 = Position::new();
        let mut pos2 = Position::new();
        pos2.offset(0,0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset(1,0);
        assert_eq!(pos2.x, pos1.x + 1);
        pos2 = Position::new();
        pos2.offset(-1,0);
        assert_eq!(pos2.x + 1, pos1.x);
        pos2 = Position::new();
        pos2.offset(BOARD_WIDTH as i8,0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset((BOARD_WIDTH as i8) + 1,0);
        assert_eq!(pos2.x, pos1.x + 1);
        pos2 = Position::new();
        pos2.offset(-(BOARD_WIDTH as i8),0);
        assert_eq!(pos2.x, pos1.x);
        pos2 = Position::new();
        pos2.offset(-(BOARD_WIDTH as i8) - 1,0);
        assert_eq!(pos2.x + 1, pos1.x);
    }
    
    #[test]
    fn test_position_new_offset() {
        let mut pos1 = Position::new();
        let mut pos2 = Position::new_offset(0,0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(1,0);
        pos2 = Position::new_offset(1,0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-1,0);
        pos2 = Position::new_offset(-1,0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(BOARD_WIDTH as i8,0);
        pos2 = Position::new_offset(BOARD_WIDTH as i8,0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(BOARD_WIDTH as i8 + 1,0);
        pos2 = Position::new_offset(BOARD_WIDTH as i8 + 1,0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-(BOARD_WIDTH as i8),0);
        pos2 = Position::new_offset(-(BOARD_WIDTH as i8),0);
        assert_eq!(pos1, pos2);
        pos1 = Position::new();
        pos1.offset(-(BOARD_WIDTH as i8) - 1,0);
        pos2 = Position::new_offset(-(BOARD_WIDTH as i8) - 1,0);
        assert_eq!(pos1, pos2);
    }
    
    #[test]
    fn test_snake_new() {
        let snake = Snake::new();
        assert_eq!(snake.body.len(),3);
        assert_eq!(snake.direction, Direction::RIGHT);
        let pos1 = Position::new();
        let pos2 = Position::new_offset(-1,0);
        let pos3 = Position::new_offset(-2,0);
        assert_eq!(snake.body[0].position, pos1);
        assert_eq!(snake.body[1].position, pos2);
        assert_eq!(snake.body[2].position, pos3);
    }
    
    #[test]
    fn test_snake_next_head_pos() {
        let mut snake = Snake::new();
        let mut next_pos = snake.next_head_pos();
        let pos = Position::new_offset(1,0);
        assert_eq!(next_pos, pos);
    }
    
    #[test]
    fn test_snake_check_collide_wall() {
        let mut snake = Snake::new();
        let mut pos = Position::new();
        assert!(snake.check_collide_wall(&pos));
        pos = Position::new_offset(1,0);
        assert!(!snake.check_collide_wall(&pos));
        /*
        pos = Position {x: BOARD_WIDTH, y: BOARD_HEIGHT};
        assert!(snake.check_collide_wall(&pos));
        pos = Position {x: BOARD_WIDTH - 1, y: BOARD_HEIGHT - 1};
        assert!(!snake.check_collide_wall(&pos));
        */
    }
    
    #[test]
    fn test_snake_check_collide_body() {
        let mut snake = Snake::new();
        let mut pos = Position::new();
        assert!(snake.check_collide_body(&pos));
        pos.offset(1,0);
        assert!(!snake.check_collide_body(&pos));
        pos.offset(-3,0);
        assert!(snake.check_collide_body(&pos));
        pos.offset(-4,0);
        assert!(!snake.check_collide_body(&pos));
    }
    
    #[test]
    fn test_snake_move_next() {
        let mut snake = Snake::new();
        snake.move_next();
        let mut pos = Position::new_offset(1,0);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-1,0);
        assert_eq!(snake.body[2].position, pos);
        pos = Position::new_offset(-2,0);
        assert!(!snake.check_collide_body(&pos));
    }
    
    #[test]
    fn test_snake_eat_next() {
        let mut snake = Snake::new();
        let mut next_pos = snake.next_head_pos();
        snake.eat_next(&mut next_pos);
        assert_eq!(snake.body.len(), 4);
        let mut pos = Position::new_offset(1,0);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-2,0);
        assert_eq!(snake.body[3].position, pos);
        pos = Position::new_offset(-3,0);
        assert!(!snake.check_collide_body(&pos));
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
        let mut food = Position::new_offset(1,0);
        snake.perform_next(&mut food);
        assert_eq!(snake.body.len(), 4);
        food = Position::new_offset(0,1);
        snake.update(Direction::UP);
        snake.perform_next(&mut food);
        let mut pos = Position::new_offset(1,-1);
        assert_eq!(snake.body[0].position, pos);
        pos = Position::new_offset(-1,0);
        assert_eq!(snake.body[3].position, pos);
        pos = Position::new_offset(-2,0);
        assert!(!snake.check_collide_body(&pos));
        // Check whether we collide with the walls and die
        while snake.body[0].position.y >= 1 {
            snake.perform_next(&mut food);
        }
        let mut next_pos = snake.next_head_pos();
        assert!(snake.check_collide_wall(&next_pos));
        assert!(snake.alive);
        snake.perform_next(&mut food);
        assert!(!snake.alive);
        // Check whether we collide with ourself and die
        snake = Snake::new();
        food = Position::new_offset(1,0);
        snake.perform_next(&mut food);
        food = Position::new_offset(2,0);
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
        pos = Position::new_offset(0,1);
        assert_eq!(snake.body[0].position, pos);
        while snake.body[0].position.y <= BOARD_HEIGHT - 2 {
            snake.perform_next(&mut food);
        }
        let mut next_pos = snake.next_head_pos();
        println!("Head: {:?}; Next: {:?}; Alive: {}", snake.body[0].position, next_pos, snake.alive);
        snake.perform_next(&mut food);
        println!("Head: {:?}; Next: {:?}; Alive: {}", snake.body[0].position, next_pos, snake.alive);
        assert!(!snake.alive);
    }
    
    #[test]
    fn test_game_new() {
        let mut game = Game::new();
        assert_eq!(game.snake.body[0].position, game.food.position);
        assert_eq!(game.snake.body.len(), 3);
        assert_eq!(game.time, 0);
        assert_eq!(game.score, 0);
    }
    
    #[test]
    fn test_game_get_food_pos() {
        let mut game = Game::new();
        for i in 0..10 {
            let mut pos = game.get_food_pos();
            assert!(!game.snake.check_collide_body(&pos));
        }
    }
    
    #[test]
    fn test_game_init() {
        let mut game = Game::new();
        game.init();
        assert!(!game.snake.check_collide_body(&game.food.position));
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
        pos.offset(1,0);
        assert_eq!(game.snake.body[0].position, pos);
    } 
    
    #[test]
    fn test_game_run() {
        let mut game = Game::new();
        game.init();
        game.run();
        assert_eq!(game.time, BOARD_WIDTH as u32/2);
    }
    
    /*
    #[test]
    fn test_game_get_nn_input() {
        let mut game = Game::new();
        game.init();
        let board = game.get_nn_input();
        assert_eq!(board.len(), (BOARD_WIDTH * BOARD_HEIGHT) as usize);
        assert_eq!(board[(game.food.position.y * BOARD_WIDTH + game.food.position.x) as usize], 1f64);
        assert_eq!(board[(game.snake.body[0].position.y * BOARD_WIDTH + game.snake.body[0].position.x) as usize], 0f64);
    }
    */
    
    #[test]
    fn test_game_get_dir_nn() {
        let mut game = Game::new();
        game.init();
        let mut nn = NN::new();
        let mut layer1 = Layer::new(8, 8).unwrap();
        let mut layer2 = Layer::new(8, 4).unwrap();
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
        let mut nn = Game::create_nn();
        assert_eq!(nn.layers.len(), 2);
        assert_eq!(nn.layers[0].num_inputs, 8);
        assert_eq!(nn.layers[1].num_neurons, 4);
    }
    
    #[test]
    fn test_game_run_nn() {
        use std::cmp;
        
        let mut game = Game::new();
        game.init();
        let mut nn = NN::new();
        let mut layer1 = Layer::new(8, 8).unwrap();
        let mut layer2 = Layer::new(8, 4).unwrap();
        nn.add(layer1);
        nn.add(layer2);
        game.run_nn(&mut nn);
        println!("{}", game.time);
        assert!(game.time >= cmp::min(BOARD_WIDTH as u32, BOARD_HEIGHT as u32)/2);
    }
    
    /*
    #[test]
    fn test_game_get_population() {
        let mut game = Game::new();
        game.init();
        let pop = game.get_population(10);
        assert_eq!(pop.nn.len(),10);
        assert_ne!(pop.nn[0].layers[0].biases, pop.nn[1].layers[0].biases);
        assert_ne!(pop.nn[0].layers[0].weights, pop.nn[1].layers[0].weights);
    }
    
    #[test]
    fn test_game_population_play() {
        use std::cmp;
        
        let mut game = Game::new();
        game.init();
        let mut pop = game.get_population(10);
        game.population_play(&mut pop, 10);
        let fits: Vec<u32> = pop.nn.iter().map(|n| n.fitness).collect();
        println!("{:?}", fits);
        assert!(true);
    }
    */
    
    #[test]
    fn test_game_get_board() {
        let mut game = Game::new();
        let board = game.get_board().board;
        assert_eq!(board[game.food.position.y as usize][game.food.position.x as usize], 0f64);
    }
    
    #[test]
    fn test_game_get_nn_inputs() {
        let mut game = Game::new();
        game.init();
        println!("****Start****");
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs.len(), 8);
        println!("****Right Food****");
        game.food.position.x = game.snake.body[0].position.x+1;
        game.food.position.y = game.snake.body[0].position.y;
        inputs = game.get_nn_inputs();
        assert_eq!(inputs[1], 1f64);
        println!("****Right Dead****");
        for _ in 0..BOARD_WIDTH/2-1 {
            game.next_tick(0.1);
        }
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        println!("****Up Right Dead****");
        game.update(Direction::UP);
        for _ in 0..BOARD_HEIGHT/2 {
            game.next_tick(0.1);
        }
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[2], 1f64);
        println!("****Up Right Dead****");
        game.update(Direction::LEFT);
        for _ in 0..BOARD_WIDTH-1 {
            game.next_tick(0.1);
        }
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[2], 1f64);
        assert_eq!(inputs[4], 1f64);
        println!("****Down Left Dead****");
        game.update(Direction::DOWN);
        for _ in 0..BOARD_HEIGHT-1 {
            game.next_tick(0.1);
        }
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs[2], 1f64);
        assert_eq!(inputs[4], 1f64);
        assert_eq!(inputs[6], 1f64);
        println!("****Down Right Dead****");
        game.update(Direction::RIGHT);
        for _ in 0..BOARD_WIDTH-1 {
            game.next_tick(0.1);
        }
        let mut inputs = game.get_nn_inputs();
        assert_eq!(inputs[0], 1f64);
        assert_eq!(inputs[4], 1f64);
        assert_eq!(inputs[6], 1f64);
        //assert!(false);
    }

}


