extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use crate::common::Point;
use crate::constants::*;
use crate::game::{Block, Brain, Direction, Game};
use crate::longest::LongestCycle;

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

    pub fn run_longest(&mut self) {
        let mut game = Game::new();
        game.init();

        let mut board = vec![vec![0_i8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
        for part in game.snake.body.iter() {
            board[part.position.y as usize][part.position.x as usize] = 1_i8;
        }

        let start = Point {
            x: game.snake.body[0].position.y as usize,
            y: game.snake.body[0].position.x as usize,
        };
        let target = Point {
            x: game.snake.body[game.snake.body.len() - 1].position.y as usize,
            y: game.snake.body[game.snake.body.len() - 1].position.x as usize,
        };
        if let Some(longest) = LongestCycle::new(
            &board,
            Point::get_index_from_point(BOARD_WIDTH as usize, &start),
            Point::get_index_from_point(BOARD_WIDTH as usize, &target),
        ) {
            if let Some(cycle) = longest.get_longest_cycle() {
                println!("Length: {:?}; Cycle: {:?}", cycle.len(), cycle);
                
                let mut current_index = 0;
                
                while let Some(e) = self.events.next(&mut self.window) {
                    if let Some(args) = e.render_args() {
                        self.render_game(&args, &game);
                    }

                    if let Some(args) = e.update_args() {
                    
                        let current_i = cycle[current_index];
                        let next_i = cycle[(current_index + cycle.len() - 1) % cycle.len()];
                        
                        let current_point = Point::get_point_from_index(BOARD_WIDTH as usize, current_i);
                        let next_point = Point::get_point_from_index(BOARD_WIDTH as usize, next_i);
                        
                        //println!("Current: {:?}, Next: {:?}", current_point, next_point);
                        
                        let mut dir = game.snake.direction;
                        if next_point.x == current_point.x + 1 {
                            dir = Direction::UP;
                        }
                        else if next_point.x + 1 == current_point.x {
                            dir = Direction::DOWN;
                        }
                        else if next_point.y == current_point.y + 1 {
                            dir = Direction::LEFT;
                        }
                        else if next_point.y + 1 == current_point.y {
                            dir = Direction::RIGHT;
                        }
                        else {
                            dir = game.snake.direction;
                        }
                    
                        current_index = (current_index + cycle.len() - 1) % cycle.len();
                        game.update(dir);
                        game.next_tick(args.dt);
                    }

                    if let Some(button) = e.press_args() {
                        self.handle_events(button, &mut game);
                        current_index = 0;
                    }
                }
                
                
            }
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
