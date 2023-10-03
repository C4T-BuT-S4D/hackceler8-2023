use pyo3::pyclass;

use crate::settings::{GameMode, Settings};

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum Move {
    NONE,
    A,
    D,
    W,
    WA,
    WD,
    S,
    SA,
    SD,
}

impl Move {
    pub fn iterator(settings: &Settings) -> std::slice::Iter<'static, Move> {
        // static DIRECTIONS: [Move; 2] = [Move::S, Move::SA];
        // DIRECTIONS.iter()

        static DIRECTIONS_SCROLLER: [Move; 9] = [
            Move::NONE,
            Move::A,
            Move::D,
            Move::W,
            Move::WA,
            Move::WD,
            Move::S,
            Move::SA,
            Move::SD,
        ];
        static DIRECTIONS_PLATFORMER: [Move; 6] =
            [Move::NONE, Move::A, Move::D, Move::W, Move::WA, Move::WD];
        match settings.mode {
            GameMode::Scroller => DIRECTIONS_SCROLLER.iter(),
            GameMode::Platformer => DIRECTIONS_PLATFORMER.iter(),
        }
    }

    pub fn is_only_vertical(&self) -> bool {
        *self == Move::S || *self == Move::W
    }
}

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum Direction {
    N,
    S,
    E,
    W,
}
