use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Color, DrawMode, Image, Mesh, Rect};
use ggez::mint;
use ggez::{Context, GameResult};
use itertools::iproduct;

use crate::board::Board;

const CURSOR_INVALID: (usize, usize) = (usize::max_value(), usize::max_value());

#[derive(Debug)]
enum Command {
    Nop,
    Erase(usize, usize),
    Reset,
    Quit,
}

#[derive(Debug)]
pub struct GameState {
    imgs_tile: Vec<Image>,
    board: Board,
    cursor: (usize, usize),
    cmd: Command,
    score: i32,
}

impl GameState {
    const BOARD_W: usize = 20;
    const BOARD_H: usize = 10;

    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let imgs_tile = (1..=5)
            .map(|i| Image::new(ctx, format!("/tile-{}.png", i)))
            .collect::<GameResult<Vec<_>>>()?;

        let board = Board::random(Self::BOARD_W, Self::BOARD_H);

        let cursor = CURSOR_INVALID;
        let cmd = Command::Nop;

        let score = 0;

        Ok(Self {
            imgs_tile,
            board,
            cursor,
            cmd,
            score,
        })
    }

    fn calc_cursor(&self, x: f32, y: f32) -> (usize, usize) {
        if x < 0.0 || y < 0.0 {
            return CURSOR_INVALID;
        }

        let cx = x as usize / 32;
        let cy = y as usize / 32;
        if cx >= self.board.width() || cy >= self.board.height() {
            return CURSOR_INVALID;
        }

        (cx, cy)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match self.cmd {
            Command::Erase(x, y) => {
                let n = self.board.erase_component(x, y);
                self.score += (n - 1).pow(2) as i32;
                eprintln!("Score: {}", self.score);
            }
            Command::Reset => {
                self.board = Board::random(Self::BOARD_W, Self::BOARD_H);
                self.score = 0;
            }
            Command::Quit => {
                event::quit(ctx);
            }
            _ => {}
        }
        self.cmd = Command::Nop;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for (x, y) in iproduct!(0..self.board.width(), 0..self.board.height()) {
            let color = self.board.at(x, y);
            if color == 0 {
                continue;
            }

            let img = &self.imgs_tile[(color - 1) as usize];
            graphics::draw(
                ctx,
                img,
                (mint::Point2 {
                    x: 32.0 * x as f32,
                    y: 32.0 * y as f32,
                },),
            )?;
        }

        if self.cursor != CURSOR_INVALID {
            let ps = self.board.calc_component(self.cursor.0, self.cursor.1);
            for (x, y) in ps {
                let mesh = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(32.0 * x as f32, 32.0 * y as f32, 32.0, 32.0),
                    Color::from_rgba(0xc0, 0xc0, 0xc0, 0x80),
                )?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button != MouseButton::Left {
            return;
        }

        let cursor = self.calc_cursor(x, y);
        if cursor != CURSOR_INVALID {
            self.cmd = Command::Erase(cursor.0, cursor.1);
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.cursor = self.calc_cursor(x, y);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        if repeat {
            return;
        }

        match keycode {
            KeyCode::Escape | KeyCode::Q => {
                self.cmd = Command::Quit;
            }
            KeyCode::R => {
                self.cmd = Command::Reset;
            }
            _ => {}
        }
    }
}
