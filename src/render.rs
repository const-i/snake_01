use crate::constants::*;
use crate::game::{Block, Brain, Direction, Game};

use piston_window::*;

pub struct Render {
    window: PistonWindow,
    events: Events,
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
        }
    }

    pub fn run(&mut self) {
        let mut game = Game::new();
        game.init();

        while let Some(e) = self.events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render_game(&args, &game, &e);
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
                self.render_game(&args, &game, &e);
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

    fn render_game(&mut self, _args: &RenderArgs, game: &Game, e: &Event) {
        self.window.draw_2d(e, |_, g, _| {
            clear([1.0; 4], g);
        });
        for b in game.snake.body.iter() {
            self.render_block(&b, e);
        }
        self.render_block(&game.food, e);
    }

    fn render_block(&mut self, block: &Block, e: &Event) {
        //args: &RenderArgs

        use graphics::math::Matrix2d;

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
        self.window.draw_2d(e, |c, g, _| {
            square_.draw(dims_, &c.draw_state, transform_, g);
        });
    }
}
