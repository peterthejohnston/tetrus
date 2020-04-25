use crate::game::{self, Tets};

type Point2 = ggez::nalgebra::Point2<i8>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetType {
    I, J, L, O, S, T, Z,
}

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

    pub fn move_left(&mut self, tets: &Tets) {
        for block in self.blocks.iter() {
            if self.pos.x + block.x == 0 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x - 1).is_some() {
                return;
            }
        }
        self.pos.x -= 1;
    }

    pub fn move_right(&mut self, tets: &Tets) {
        for block in self.blocks.iter() {
            if self.pos.x + block.x + 1 >= game::TILES_WIDE as i8 ||
               tets.at(self.pos.y + block.y, self.pos.x + block.x + 1).is_some() {
                return;
            }
        }
        self.pos.x += 1;
    }

    pub fn rotate_c(&mut self, tets: &Tets) {
        // TODO: check if can rotate
        // if not, kick?
        match self.tet_type {
            TetType::O => (),
            TetType::I => {
                // rotate around center of a 4x4
                for block in self.blocks.iter_mut() {
                    let y = block.x;
                    block.x = (3 - block.y as i8).abs() as i8;
                    block.y = y;
                }
            },
            _ => {
                // rotate around center of a 3x3
                for block in self.blocks.iter_mut() {
                    let y = block.x;
                    block.x = (2 - block.y as i8).abs() as i8;
                    block.y = y;
                }
            }
        }
    }
}
