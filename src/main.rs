extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;


// The game class
pub struct Game {
    gl: GlGraphics,
    rows: u32,
    cols: u32,
    snake: Snake,
    just_eaten: bool,
    square_width: u32,
    food: Food,
    score: u32,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        const BLACK: [f32; 4] = [0.0; 4];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(BLACK, gl);
        });

        self.snake.render(args);
        self.food.render(&mut self.gl, args, self.square_width);
    }

    fn update(&mut self, _args: &UpdateArgs) -> bool {
        if !self.snake.update(self.just_eaten, self.cols, self.rows) {
            return false;
        }

        if self.just_eaten {
            self.score += 1;
            self.just_eaten = false;
        }

        self.just_eaten = self.food.update(&self.snake);
        if self.just_eaten {
            use rand::Rng;
            use rand::thread_rng;

            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0..self.cols);
                let new_y = r.gen_range(0..self.rows);
                if !self.snake.is_collide(new_x, new_y) {
                    self.food = Food { x: new_x, y: new_y };
                    break;
                }
            }
        }

        true
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.dir.clone();
        self.snake.dir = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            _ => last_direction,
        };
    }
}

// A simple enum to represent analog directions.
#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Left,
    Right,
    Down,
    Still,
}

//The snake class
pub struct Snake {
    gl: GlGraphics,
    body: LinkedList<SnakePiece>,
    width: u32,
    dir: Direction,
}

#[derive(Clone)]
pub struct SnakePiece(u32, u32);   

impl Snake {
    pub fn render(&mut self, args: &RenderArgs) {

        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        //const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|p| SnakePiece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(BLUE, square, transform, gl));
        })
    }

    pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_head: SnakePiece = 
            (*self.body.front().expect("Snake has no body!")).clone();
        
        if (self.dir == Direction::Up && new_head.1 == 0)
        || (self.dir == Direction::Left && new_head.0 == 0)
        || (self.dir == Direction::Right && new_head.0 == cols - 1)
        || (self.dir == Direction::Down && new_head.1 == rows -1)
        {
            return false;
        }

        match self.dir {
            Direction::Up => new_head.1 -= 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Down => new_head.1 += 1,
            Direction::Still => new_head = new_head,
        }

        if !just_eaten {
            self.body.pop_back();
        }

        if self.is_collide(new_head.0, new_head.1) {
            return false;
        }

        self.body.push_front(new_head);
        true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {
        self.body.iter().any(|p| x == p.0 && y == p.1)
    }
}

pub struct Food {
    x: u32,
    y: u32,
}

impl Food {
    // Return true if snake ate food this update
    fn update(&mut self, snake: &Snake) -> bool {
        let front = snake.body.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
        const WHITE: [f32; 4] = [1.0; 4];

        let x = self.x * width;
        let y = self.y * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(WHITE, square, transform, gl)
        });
    }
}

fn main() {
    let opengl = OpenGL::V4_5;

    const COLS: u32 = 30;
    const ROWS: u32 = 20;
    const SQUARE_WIDTH: u32 = 20;

    const WIDTH: u32 = COLS * SQUARE_WIDTH;
    const HEIGHT: u32 = ROWS * SQUARE_WIDTH;

    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLS,
        square_width: SQUARE_WIDTH,
        just_eaten: false,
        food: Food { x: 1, y: 1 },
        score: 0,
        snake: Snake {
            gl: GlGraphics::new(opengl),
            body: LinkedList::from_iter((vec![SnakePiece(COLS / 2, ROWS / 2)]).into_iter()),
            width: SQUARE_WIDTH,
            dir: Direction::Still,
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(15);

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
    println!("Congratulations, your score was: {}", game.score);
}
