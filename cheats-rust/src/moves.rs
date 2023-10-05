use pyo3::pyclass;

use crate::settings::{GameMode, SearchSettings};

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
    pub fn all(settings: &SearchSettings) -> Vec<Move> {
        if !settings.allowed_moves.is_empty() {
            return settings.allowed_moves.clone();
        }

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
            GameMode::Scroller => DIRECTIONS_SCROLLER.into(),
            GameMode::Platformer => DIRECTIONS_PLATFORMER.into(),
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
