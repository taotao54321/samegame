use std::io::{self, BufRead};
use std::iter;
use std::mem;

use anyhow::{Context, Result};
use itertools::iproduct;
use rand::prelude::*;

const CELL_NB: u8 = 6;

// ```
// ^^^^^^E
// ||||||^
// |||||||
// S||||||
//
// S:Start, E:End
// ```
#[derive(Debug)]
pub struct Board {
    w: usize,
    h: usize,
    cells: Vec<u8>,
}

impl Board {
    pub fn random(w: usize, h: usize) -> Self {
        assert!(w > 0);
        assert!(h > 0);

        let mut rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::new(1, CELL_NB);
        let cells: Vec<_> = iter::repeat_with(|| dist.sample(&mut rng))
            .take(w * h)
            .collect();

        Self { w, h, cells }
    }

    // ```
    // 4 3
    // 0123
    // 1234
    // 2345
    // ```
    pub fn parse<R: io::Read>(rdr: R) -> Result<Self> {
        let mut rdr = io::BufReader::new(rdr);
        let mut read_line = || -> Result<String> {
            let mut line = String::new();
            rdr.read_line(&mut line)?;
            Ok(line)
        };

        let (w, h) = {
            let line = read_line()?;
            let mut it = line.split_ascii_whitespace();
            let w: usize = it.next().context("format error")?.parse()?;
            let h: usize = it.next().context("format error")?.parse()?;
            anyhow::ensure!(it.next().is_none(), "format error");
            (w, h)
        };
        anyhow::ensure!(w > 0, "w must be positive");
        anyhow::ensure!(h > 0, "h must be positive");

        let mut cells = vec![0_u8; w * h];
        let mut lines = rdr.lines();
        for y in 0..h {
            let line = lines.next().context("incomplete input")??;
            for (x, c) in line.chars().enumerate() {
                anyhow::ensure!(('0'..='5').contains(&c), "invalid char");
                let i = Self::xy2idx_h(h, x, y);
                cells[i] = c.to_digit(10).expect("internal error") as u8;
            }
        }

        Ok(Self { w, h, cells })
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.h
    }

    pub fn at(&self, x: usize, y: usize) -> u8 {
        let i = self.xy2idx(x, y);
        self.cells[i]
    }

    fn replace(&mut self, x: usize, y: usize, color: u8) -> u8 {
        let i = self.xy2idx(x, y);
        mem::replace(&mut self.cells[i], color)
    }

    pub fn calc_component(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        if self.at(x, y) == 0 {
            return vec![];
        }

        let mut res = vec![];
        let mut done = vec![false; self.w * self.h];

        fn rec(this: &Board, v: &mut Vec<(usize, usize)>, done: &mut [bool], x: usize, y: usize) {
            v.push((x, y));
            done[this.xy2idx(x, y)] = true;

            for (xx, yy) in this.neighbor(x, y) {
                if done[this.xy2idx(xx, yy)] {
                    continue;
                }
                if this.at(x, y) != this.at(xx, yy) {
                    continue;
                }
                rec(this, v, done, xx, yy);
            }
        }
        rec(self, &mut res, &mut done, x, y);

        if res.len() == 1 {
            return vec![];
        }
        res
    }

    pub fn is_finished(&self) -> bool {
        for (x, y) in iproduct!(0..self.w, 0..self.h) {
            if self.at(x, y) == 0 {
                continue;
            }
            if x > 0 && self.at(x, y) == self.at(x - 1, y) {
                return false;
            }
            if y > 0 && self.at(x, y) == self.at(x, y - 1) {
                return false;
            }
        }
        true
    }

    pub fn erase_component(&mut self, x: usize, y: usize) -> usize {
        let color = self.at(x, y);
        if color == 0 {
            return 0;
        }

        fn rec(this: &mut Board, x: usize, y: usize) -> usize {
            let color = this.replace(x, y, 0);

            let mut res = 1;
            for (xx, yy) in this.neighbor(x, y) {
                if this.at(xx, yy) != color {
                    continue;
                }
                res += rec(this, xx, yy);
            }
            res
        }
        let res = rec(self, x, y);

        if res == 1 {
            self.replace(x, y, color);
            return 0;
        }

        self.pack_cellwise();
        self.pack_colwise();

        res
    }

    // セル単位での詰め直し(各列について落下処理)
    fn pack_cellwise(&mut self) {
        for col in self.cells.chunks_exact_mut(self.h) {
            // stable_partition
            let mut i = 0;
            for j in 0..self.h {
                if col[j] != 0 {
                    col.swap(i, j);
                    i += 1;
                }
            }
        }
    }

    // 列単位での詰め直し(空になった列を詰める)
    fn pack_colwise(&mut self) {
        let mut x_target = 0;
        for x in 0..self.w {
            let i_target = self.xy2idx(x_target, self.h - 1);
            let (vl, vr) = {
                let i = self.xy2idx(x, self.h - 1);
                self.cells.split_at_mut(i)
            };

            let col = &mut vr[..self.h];
            let empty = col.iter().all(|&color| color == 0);
            if !empty {
                if x_target != x {
                    let col_target = &mut vl[i_target..i_target + self.h];
                    col_target.copy_from_slice(col);
                    for i in 0..self.h {
                        col[i] = 0;
                    }
                }
                x_target += 1;
            }
        }
    }

    fn neighbor(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut res = vec![];
        if x > 0 {
            res.push((x - 1, y));
        }
        if x < self.w - 1 {
            res.push((x + 1, y));
        }
        if y > 0 {
            res.push((x, y - 1));
        }
        if y < self.h - 1 {
            res.push((x, y + 1));
        }
        res
    }

    fn xy2idx(&self, x: usize, y: usize) -> usize {
        Self::xy2idx_h(self.h, x, y)
    }

    fn xy2idx_h(h: usize, x: usize, y: usize) -> usize {
        h * x + (h - 1 - y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random() {
        let board = Board::random(3, 14);
        assert_eq!(board.width(), 3);
        assert_eq!(board.height(), 14);
        assert!(board.cells.iter().all(|color| (1..=5).contains(color)));
    }

    #[test]
    fn parse() {
        let board = Board::parse(
            b"\
4 3
0123
1234
2345
"
            .as_ref(),
        )
        .unwrap();
        assert_eq!(board.width(), 4);
        assert_eq!(board.height(), 3);
        assert_eq!(board.cells, [2, 1, 0, 3, 2, 1, 4, 3, 2, 5, 4, 3]);
    }

    #[test]
    fn component() {
        let mut board = Board::parse(
            b"\
4 3
2102
1154
5135
"
            .as_ref(),
        )
        .unwrap();

        assert_eq!(board.calc_component(0, 0), []);
        assert_eq!(
            itertools::sorted(board.calc_component(0, 1)).collect::<Vec<_>>(),
            [(0, 1), (1, 0), (1, 1), (1, 2)]
        );
        assert!(!board.is_finished());

        assert_eq!(board.erase_component(0, 0), 0);
        assert_eq!(board.erase_component(3, 0), 0);
        assert_eq!(board.erase_component(1, 1), 4);
        assert_eq!(board.cells, [5, 2, 0, 3, 5, 0, 5, 4, 2, 0, 0, 0]);
        assert!(board.is_finished());
    }
}
