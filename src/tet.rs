use rand::{Rng, distributions::{Distribution, Standard}};

use crate::game::{self, Tets};

type Point2 = ggez::nalgebra::Point2<i8>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetType {
    I, J, L, O, S, T, Z,
}

impl Distribution<TetType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TetType {
        match rng.gen_range(0, 7) {
            0 => TetType::I,
            1 => TetType::J,
            2 => TetType::L,
            3 => TetType::O,
            4 => TetType::S,
            5 => TetType::T,
            _ => TetType::Z,
        }
    }
}

#[derive(Clone)]
pub struct Tet {
    pub tet_type: TetType,
    pub blocks: [Point2; 4],
    pub pos: Point2,
}

impl Tet {
    pub fn new(tet_type: TetType, pos: Point2) -> Self {
        Self {
            tet_type,
            pos,
            blocks: match tet_type {
                TetType::I => [[0, 1].into(), [1, 1].into(), [2, 1].into(), [3, 1].into()],
                TetType::J => [[0, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
                TetType::L => [[0, 1].into(), [1, 1].into(), [2, 1].into(), [2, 0].into()],
                TetType::O => [[0, 0].into(), [0, 1].into(), [1, 0].into(), [1, 1].into()],
                TetType::S => [[0, 1].into(), [1, 1].into(), [1, 0].into(), [2, 0].into()],
                TetType::T => [[1, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
                TetType::Z => [[0, 0].into(), [1, 0].into(), [1, 1].into(), [2, 1].into()],
            }
        }
    }

    fn left(&self) -> i8 {
        self.blocks.iter().min_by(|b1, b2| b1.x.cmp(&b2.x)).unwrap().x
    }

    fn right(&self) -> i8 {
        self.blocks.iter().max_by(|b1, b2| b1.x.cmp(&b2.x)).unwrap().x
    }

    fn bottom(&self) -> i8 {
        self.blocks.iter().max_by(|b1, b2| b1.y.cmp(&b2.y)).unwrap().y
    }

    pub fn at_bottom(&self, tets: &Tets) -> bool {
        for block in self.blocks.iter() {
            if self.pos.y + block.y + 1 >= game::TILES_HIGH as i8 ||
               tets.at(self.pos.y + block.y + 1, self.pos.x + block.x).is_some() {
                return true;
            }
        }
        false
    }

    pub fn fall(&mut self, tets: &Tets) -> bool {
        for block in self.blocks.iter() {
            if self.pos.y + block.y + 1 >= game::TILES_HIGH as i8 ||
               tets.at(self.pos.y + block.y + 1, self.pos.x + block.x).is_some() {
                return false;
            }
        }
        self.pos.y += 1;
        true
    }

    pub fn move_left(&mut self, tets: &Tets) -> bool {
        for block in self.blocks.iter() {
            if self.pos.x + block.x == 0 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x - 1).is_some() {
                return false;
            }
        }
        self.pos.x -= 1;
        true
    }

    pub fn move_right(&mut self, tets: &Tets) -> bool {
        for block in self.blocks.iter() {
            if self.pos.x + block.x + 1 >= game::TILES_WIDE as i8 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x + 1).is_some() {
                return false;
            }
        }
        self.pos.x += 1;
        true
    }

    pub fn rotate_c(&mut self, tets: &Tets) -> bool {
        if let TetType::O = self.tet_type {
            return true;
        }
        let moved_blocks: Vec<_> = match self.tet_type {
            TetType::I => {
                // rotate around center of a 4x4
                self.blocks.iter().map(|block| {
                    Point2::new((3 - block.y as i8).abs() as i8, block.x)
                }).collect()
            },
            _ => {
                // rotate around center of a 3x3
                self.blocks.iter().map(|block| {
                    Point2::new((2 - block.y as i8).abs() as i8, block.x)
                }).collect()
            },
        };
        // check for collisions/blocks past edge of screen
        // TODO: we might not need to be so comprehensive
        // TODO: kicking
        for block in &moved_blocks {
            let x = self.pos.x + block.x;
            let y = self.pos.y + block.y;
            if x < 0 || x >= game::TILES_WIDE as i8 || y >= game::TILES_HIGH as i8 ||
                tets.at(y, x).is_some() {
                return false;
            }
        }
        for (i, block) in moved_blocks.iter().enumerate() {
            self.blocks[i] = *block;
        }
        true
    }
}
