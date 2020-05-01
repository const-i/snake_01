
extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

use crate::constants::*;
use crate::game::{Game, Block, Direction};
use crate::brain::{NN, Layer};

use piston::window::{WindowSettings, Window};
use piston::event_loop::{Events, EventSettings, EventLoop};
use piston::input::{Button, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, PressEvent};
use piston::input::keyboard::Key;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

const BLOCK_SIZE: u32 = 25;
const RENDER_UPS: u64 = 20;
const RENDER_FPS_MAX: u64 = 20;

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
                                [
                                    BOARD_WIDTH as u32 * BLOCK_SIZE,
                                    BOARD_HEIGHT as u32 * BLOCK_SIZE
                                ])
                                .graphics_api(OpenGL::V3_2)
                                .vsync(true)
                                .exit_on_esc(true)
                                .build()
                                .unwrap(),
            events: Events::new(EventSettings::new()
                                .ups(RENDER_UPS)
                                .max_fps(RENDER_FPS_MAX)),
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
    
    pub fn run_nn(&mut self, nn: &NN) {
        
        let mut game = Game::new();
        game.init();
    
        while let Some(e) = self.events.next(&mut self.window) {

            if let Some(args) = e.render_args() {
                self.render_game(&args, &game);
            }

            if let Some(args) = e.update_args() {
                let dir = game.get_dir_nn(&nn);
                game.update(dir);
                game.next_tick(args.dt);
            }

            if let Some(button) = e.press_args() {
                self.handle_events(button, &mut game);
            }
        } 
    }
    
    fn handle_events(&mut self, button: Button, game: &mut Game) {
        match button {
            Button::Keyboard(key) => {
                match key {
                    Key::Up    => game.update(Direction::UP),
                    Key::Down  => game.update(Direction::DOWN),
                    Key::Left  => game.update(Direction::LEFT),
                    Key::Right => game.update(Direction::RIGHT),
                    Key::Space => game.init(),
                    _ => return
                }
            },
            _ => {}
        }
    }
    
    fn render_game(&mut self, args: &RenderArgs, game: &Game) {
        self.gl.draw(args.viewport(), |c, g| {
            graphics::clear(BLACK, g);
        });
        for b in game.snake.body.iter() {
            self.render_block(&b);
        }
        self.render_block(&game.food);
    }
    
    fn render_block(&mut self, block: &Block) { //args: &RenderArgs
    
        use graphics::Transformed;
        use graphics::math::Matrix2d;
        
        
        let square_ = graphics::rectangle::Rectangle::new(block.colour)
                        .border(graphics::rectangle::Border {color: BLACK, radius: 0.01});
        let dims_ = graphics::rectangle::rectangle_by_corners(0.0, 0.0, 
                                                   2.0 / BOARD_WIDTH as f64, 
                                                   2.0 / BOARD_HEIGHT as f64);
        let transform_: Matrix2d = graphics::math::identity()
                        .trans(-((BOARD_WIDTH/2) as f64) * 2.0 / BOARD_WIDTH as f64,
                                (BOARD_HEIGHT/2-1) as f64 * 2.0 / BOARD_HEIGHT as f64)
                        .trans((block.position.x as f64) * 2.0 / BOARD_WIDTH as f64,
                              -(block.position.y as f64) * 2.0 / BOARD_HEIGHT as f64);
        let draw_state_ = graphics::draw_state::DrawState::default();
        square_.draw(dims_, &draw_state_, transform_, &mut self.gl);
        
    }
    
}


/*
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_new() {
        let render = Render::new();
        assert_eq!(render.window.size().width.round(), BOARD_WIDTH as f64 * BLOCK_SIZE as f64);
        assert_eq!(render.window.size().height.round(), BOARD_HEIGHT as f64 * BLOCK_SIZE as f64);
    }
    
    
    /*
    #[test]
    fn test_render_render_block() {
        let mut render = Render::new();
        let mut game = Game::new();
        game.init();
        render.render_block(&game.food);
        assert!(true);
    }
    
    #[test]
    fn test_render_run() {
        let mut render = Render::new();
        render.run();
        assert!(true);
    }
    */
    
}
*/
