use rand::seq::SliceRandom;

use crate::game::{self, Tets};

type Point2 = ggez::nalgebra::Point2<i8>;
type Point2f32 = ggez::nalgebra::Point2<f32>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetType {
    I, J, L, O, S, T, Z,
}

impl TetType {
    pub fn blocks(&self) -> [Point2; 4] {
        match self {
            TetType::I => [[0, 1].into(), [1, 1].into(), [2, 1].into(), [3, 1].into()],
            TetType::J => [[0, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
            TetType::L => [[0, 1].into(), [1, 1].into(), [2, 1].into(), [2, 0].into()],
            TetType::O => [[0, 0].into(), [0, 1].into(), [1, 0].into(), [1, 1].into()],
            TetType::S => [[0, 1].into(), [1, 1].into(), [1, 0].into(), [2, 0].into()],
            TetType::T => [[1, 0].into(), [0, 1].into(), [1, 1].into(), [2, 1].into()],
            TetType::Z => [[0, 0].into(), [1, 0].into(), [1, 1].into(), [2, 1].into()],
        }
    }

    // Offset needed to center the piece in a 4x4 tile size area, in its
    // initial rotation (used for display in holding and preview areas)
    pub fn center_4x4(&self) -> Point2f32 {
        match self {
            TetType::I => [0.0, 0.5].into(),
            TetType::O => [1.0, 1.0].into(),
            _ => [0.5, 1.0].into(),
        }
    }
}

impl TetType {
    pub fn batch() -> [TetType; 7] {
        let mut rng = rand::thread_rng();
        let mut batch = [
            TetType::I,
            TetType::J,
            TetType::L,
            TetType::O,
            TetType::S,
            TetType::T,
            TetType::Z
        ];
        batch.shuffle(&mut rng);
        batch
    }
}

pub enum RotationDir {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Debug)]
enum Rot {
    Zero = 0,
    R    = 1,
    Two  = 2,
    L    = 3,
}

impl Rot {
    fn c(&mut self) {
        *self = match self {
            Rot::Zero => Rot::R,
            Rot::R    => Rot::Two,
            Rot::Two  => Rot::L,
            Rot::L    => Rot::Zero,
        }
    }

    fn cc(&mut self) {
        *self = match self {
            Rot::Zero => Rot::L,
            Rot::R    => Rot::Zero,
            Rot::Two  => Rot::R,
            Rot::L    => Rot::Two,
        }
    }
}

const C_KICKS: [[[i8; 2]; 5]; 4] = [
    [[0, 0], [-1, 0], [-1, -1], [0,  2], [-1,  2]], // 0 -> R
    [[0, 0], [ 1, 0], [ 1,  1], [0, -2], [ 1, -2]], // R -> 2
    [[0, 0], [ 1, 0], [ 1, -1], [0,  2], [ 1,  2]], // 2 -> L
    [[0, 0], [-1, 0], [-1,  1], [0, -2], [-1, -2]], // L -> 0
];

const C_I_KICKS: [[[i8; 2]; 5]; 4] = [
    [[0, 0], [-2, 0], [ 1, 0], [-2,  1], [ 1, -2]], // 0 -> R
    [[0, 0], [-1, 0], [ 2, 0], [-1, -2], [ 2,  1]], // R -> 2
    [[0, 0], [ 2, 0], [-1, 0], [ 2, -1], [-1,  2]], // 2 -> L
    [[0, 0], [ 1, 0], [-2, 0], [ 1,  2], [-2, -1]], // L -> 0
];

const CC_KICKS: [[[i8; 2]; 5]; 4] = [
    [[0, 0], [ 1, 0], [ 1, -1], [0,  2], [ 1,  2]], // 0 -> L
    [[0, 0], [-1, 0], [-1,  1], [0, -2], [ 1, -2]], // L -> 2
    [[0, 0], [-1, 0], [-1, -1], [0,  2], [-1,  2]], // 2 -> R
    [[0, 0], [ 1, 0], [ 1,  1], [0, -2], [ 1, -2]], // R -> 0
];

const CC_I_KICKS: [[[i8; 2]; 5]; 4] = [
    [[0, 0], [-1, 0], [ 2, 0], [-1, -2], [ 2,  1]], // 0 -> L
    [[0, 0], [-2, 0], [ 1, 0], [-2,  1], [ 1, -2]], // L -> 2
    [[0, 0], [ 1, 0], [-2, 0], [ 1,  2], [-2, -1]], // 2 -> R
    [[0, 0], [ 2, 0], [-1, 0], [ 2, -1], [-1,  2]], // R -> 0
];

#[derive(Clone)]
pub struct Tet {
    pub tet_type: TetType,
    pub blocks: [Point2; 4],
    pub pos: Point2,
    rot: Rot,
}

impl Tet {
    pub fn new(tet_type: TetType, pos: Point2) -> Self {
        Self {
            tet_type,
            pos,
            blocks: tet_type.blocks(),
            rot: Rot::Zero,
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

    fn rotate_blocks(&self, dir: &RotationDir) -> Vec<Point2> {
        match dir {
            RotationDir::Clockwise => {
                match self.tet_type {
                    TetType::I => {
                        // rotate around center of a 4x4
                        self.blocks.iter().map(|block| {
                            Point2::new((3 - block.y as i8).abs() as i8, block.x)
                        }).collect()
                    }
                    _ => {
                        // rotate around center of a 3x3
                        self.blocks.iter().map(|block| {
                            Point2::new((2 - block.y as i8).abs() as i8, block.x)
                        }).collect()
                    }
                }
            }
            RotationDir::CounterClockwise => {
                match self.tet_type {
                    TetType::I => {
                        // rotate around center of a 4x4
                        self.blocks.iter().map(|block| {
                            Point2::new(block.y, (3 - block.x as i8).abs() as i8)
                        }).collect()
                    }
                    _ => {
                        // rotate around center of a 3x3
                        self.blocks.iter().map(|block| {
                            Point2::new(block.y, (2 - block.x as i8).abs() as i8)
                        }).collect()
                    }
                }
            }
        }
    }

    pub fn rotate(&mut self, dir: RotationDir, tets: &Tets) -> bool {
        if let TetType::O = self.tet_type {
            return true;
        }
        let moved_blocks = self.rotate_blocks(&dir);
        // check for collisions/blocks past edge of screen
        let can_move = |dx: i8, dy: i8| {
            for block in &moved_blocks {
                let x = self.pos.x + block.x + dx;
                let y = self.pos.y + block.y + dy;
                if x < 0 || x >= game::TILES_WIDE as i8 || y >= game::TILES_HIGH as i8 ||
                    tets.at(y, x).is_some() {
                    return false;
                }
            }
            true
        };
        let tests = match dir {
            RotationDir::Clockwise => {
                if let TetType::I = self.tet_type { &C_I_KICKS } else { &C_KICKS }
            }
            RotationDir::CounterClockwise => {
                if let TetType::I = self.tet_type { &CC_I_KICKS } else { &CC_KICKS }
            }
        };
        for test in tests[self.rot as usize].iter() {
            if can_move(test[0], test[1]) {
                for (i, block) in moved_blocks.iter().enumerate() {
                    self.blocks[i] = *block;
                }
                self.pos.x += test[0];
                self.pos.y += test[1];
                self.rot.c();
                return true;
            }
        }
        false
    }
}
