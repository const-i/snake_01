extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use crate::constants::*;
use crate::game::{Block, Brain, Direction, Game};
use crate::pathfind::{AStar, Hamiltonian};

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::keyboard::Key;
use piston::input::{Button, PressEvent, RenderArgs, RenderEvent, UpdateEvent}; //UpdateArgs
use piston::window::WindowSettings;

pub struct Render {
    window: GlutinWindow,
    events: Events,
    gl: GlGraphics,
}

impl Render {
    pub fn new() -> Render {
        Render {
            window: WindowSettings::new(
                NAME,
                [BOARD_WIDTH as u32 * BLOCK_SIZE, BOARD_HEIGHT as u32 * BLOCK_SIZE],
            )
            .graphics_api(OpenGL::V3_2)
            .vsync(true)
            .exit_on_esc(true)
            .build()
            .unwrap(),
            events: Events::new(EventSettings::new().ups(RENDER_UPS).max_fps(RENDER_FPS_MAX)),
            gl: GlGraphics::new(OpenGL::V3_2),
        }
    }

    pub fn run(&mut self) {
        let mut game = Game::new();
        game.init();

        while let Some(e) = self.events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render_game(&args, &game);
            }

            if let Some(args) = e.update_args() {
                game.next_tick(args.dt);
            }

            if let Some(button) = e.press_args() {
                self.handle_events(button, &mut game);
            }
        }
    }

    pub fn run_brain<T: Brain>(&mut self, brain: &mut T) {
        let mut game = Game::new();
        game.init();

        while let Some(e) = self.events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render_game(&args, &game);
            }

            if let Some(args) = e.update_args() {
                let dir = game.get_dir_from_brain(brain);
                game.update(dir);
                game.next_tick(args.dt);
            }

            if let Some(button) = e.press_args() {
                self.handle_events(button, &mut game);
            }
        }
    }

    pub fn run_astar(&mut self) {
        let mut game = Game::new();
        game.init();

        while let Some(e) = self.events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render_game(&args, &game);
            }

            if let Some(args) = e.update_args() {
                let mut board = vec![vec![0; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize];
                for i in 0..game.snake.body.len() {
                    board[game.snake.body[i].position.x as usize][game.snake.body[i].position.y as usize] = 1;
                }

                let mut astar = AStar::new(game.snake.body[0].position, game.food.position, board);
                astar.calc_path();
                let path = astar.get_path();
                let mut dir = game.snake.direction;
                if path.len() > 1 {
                    let mut head = game.snake.body[0].position.clone();
                    head.offset(1, 0);
                    if path[1] == head {
                        dir = Direction::RIGHT
                    }
                    head = game.snake.body[0].position.clone();
                    head.offset(0, -1);
                    if path[1] == head {
                        dir = Direction::UP
                    }
                    head = game.snake.body[0].position.clone();
                    head.offset(-1, 0);
                    if path[1] == head {
                        dir = Direction::LEFT
                    }
                    head = game.snake.body[0].position.clone();
                    head.offset(0, 1);
                    if path[1] == head {
                        dir = Direction::DOWN
                    }
                }

                game.update(dir);
                game.next_tick(args.dt);
            }

            if let Some(button) = e.press_args() {
                self.handle_events(button, &mut game);
            }
        }
    }
    
    pub fn run_hamiltonian(&mut self) {
        let mut game = Game::new();
        game.init();
        let start: usize = game.snake.body[0].position.y as usize * BOARD_WIDTH as usize + game.snake.body[0].position.x as usize;
        let mut ham = Hamiltonian::new(BOARD_WIDTH as usize, BOARD_HEIGHT as usize, start);
        let path_maybe = ham.calc_hamiltonian_cycle();
        
        println!("{:?}", path_maybe);
        //assert!(false);
        
        if path_maybe.is_some() {
            let path = path_maybe.unwrap();
            let mut index = 0;
            let mut dir = game.snake.direction;
            
            
            while let Some(e) = self.events.next(&mut self.window) {
                if let Some(args) = e.render_args() {
                    self.render_game(&args, &game);
                }

                if let Some(args) = e.update_args() {
                    let this_pos = path[index];
                    let next_pos = path[(index + path.len() - 1) % path.len()];
                    
                    //println!("This: {:?}; Next: {:?}", this_pos, next_pos);
                    
                    
                    if next_pos == this_pos + BOARD_WIDTH as usize {
                        dir = Direction::DOWN;
                    }
                    else if next_pos == this_pos + 1 {
                        dir = Direction::RIGHT;
                    }
                    else if next_pos == this_pos - 1 {
                        dir = Direction::LEFT;
                    }

                    else if next_pos == this_pos - BOARD_WIDTH as usize {
                        dir = Direction::UP;
                    }
                    else {
                        dir = game.snake.direction;
                    }
                    
                    
                    index = (index + path.len() - 1) % path.len();
                    game.update(dir);
                    let pos1 = game.snake.body[0].position.clone();
                    game.next_tick(args.dt);
                    let pos2 = game.snake.body[0].position.clone();
                    //assert_ne!(pos1, pos2);
                    
                }

                if let Some(button) = e.press_args() {
                    self.handle_events(button, &mut game);
                }
            }
        }
        else {
            self.run();
        }

    }

    fn handle_events(&mut self, button: Button, game: &mut Game) {
        match button {
            Button::Keyboard(key) => match key {
                Key::Up => game.update(Direction::UP),
                Key::Down => game.update(Direction::DOWN),
                Key::Left => game.update(Direction::LEFT),
                Key::Right => game.update(Direction::RIGHT),
                Key::Space => game.init(),
                _ => {}
            },
            _ => {}
        }
    }

    fn render_game(&mut self, args: &RenderArgs, game: &Game) {
        self.gl.draw(args.viewport(), |_c, g| {
            graphics::clear(BLACK, g);
        });
        for b in game.snake.body.iter() {
            self.render_block(&b);
        }
        self.render_block(&game.food);
    }

    fn render_block(&mut self, block: &Block) {
        //args: &RenderArgs

        use graphics::math::Matrix2d;
        use graphics::Transformed;

        let square_ = graphics::rectangle::Rectangle::new(block.colour).border(graphics::rectangle::Border {
            color: BLACK,
            radius: 0.01,
        });
        let dims_ =
            graphics::rectangle::rectangle_by_corners(0.0, 0.0, 2.0 / BOARD_WIDTH as f64, 2.0 / BOARD_HEIGHT as f64);
        let transform_: Matrix2d = graphics::math::identity()
            .trans(
                -((BOARD_WIDTH / 2) as f64) * 2.0 / BOARD_WIDTH as f64,
                (BOARD_HEIGHT / 2 - 1) as f64 * 2.0 / BOARD_HEIGHT as f64,
            )
            .trans(
                (block.position.x as f64) * 2.0 / BOARD_WIDTH as f64,
                -(block.position.y as f64) * 2.0 / BOARD_HEIGHT as f64,
            );
        let draw_state_ = graphics::draw_state::DrawState::default();
        square_.draw(dims_, &draw_state_, transform_, &mut self.gl);
    }
}
