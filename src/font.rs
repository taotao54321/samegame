use std::path::Path;

use ggez::error::GameError::FontError;
use ggez::graphics::{self, DrawParam, Image, Rect};
use ggez::mint;
use ggez::{Context, GameResult};

#[derive(Debug)]
pub struct Font {
    img: Image,
}

impl Font {
    const NCOL: usize = 16;
    const NROW: usize = 6;

    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> GameResult<Self> {
        let img = Image::new(ctx, path)?;

        if img.width() as usize % Self::NCOL != 0 {
            return Err(FontError("width is not divisible".to_owned()));
        }
        if img.height() as usize % Self::NROW != 0 {
            return Err(FontError("height is not divisible".to_owned()));
        }

        Ok(Self { img })
    }

    pub fn glyph_width(&self) -> usize {
        self.img.width() as usize / Self::NCOL
    }

    pub fn glyph_height(&self) -> usize {
        self.img.height() as usize / Self::NROW
    }

    pub fn draw_char(&self, ctx: &mut Context, x: f32, y: f32, ch: char) -> GameResult {
        assert!(('\x20'..='\x7E').contains(&ch));

        let (ch_c, ch_r) = {
            let idx = ch as usize - 0x20;
            (idx % Self::NCOL, idx / Self::NCOL)
        };

        let src_coord = |x: usize, y: usize| -> (f32, f32) {
            (
                x as f32 / self.img.width() as f32,
                y as f32 / self.img.height() as f32,
            )
        };
        let (sx, sy) = src_coord(ch_c * self.glyph_width(), ch_r * self.glyph_height());
        let (sw, sh) = src_coord(self.glyph_width(), self.glyph_height());

        graphics::draw(
            ctx,
            &self.img,
            DrawParam::default()
                .src(Rect {
                    x: sx,
                    y: sy,
                    w: sw,
                    h: sh,
                })
                .dest(mint::Point2 { x, y }),
        )
    }

    pub fn draw_str<S: AsRef<str>>(&self, ctx: &mut Context, x: f32, y: f32, s: S) -> GameResult {
        let s = s.as_ref();

        for (i, ch) in s.chars().enumerate() {
            let dx = (i * self.glyph_width()) as f32;
            self.draw_char(ctx, x + dx, y, ch)?;
        }

        Ok(())
    }
}
