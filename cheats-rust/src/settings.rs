use pyo3::{pyclass, pymethods};

use crate::moves::Move;

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum GameMode {
    Scroller,
    Platformer,
}

#[pyclass]
#[derive(Debug, Copy, Clone)]
pub struct PhysicsSettings {
    pub mode: GameMode,
    pub enable_vpush: bool,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SearchSettings {
    pub mode: GameMode,
    pub timeout: u64,
    pub always_shift: bool,
    pub disable_shift: bool,
    pub allowed_moves: Vec<Move>,
    pub enable_vpush: bool,
}

#[pymethods]
impl SearchSettings {
    #[new]
    pub fn new(
        mode: GameMode,
        timeout: u64,
        always_shift: bool,
        disable_shift: bool,
        allowed_moves: Vec<Move>,
        enable_vpush: bool,
    ) -> Self {
        Self {
            mode,
            timeout,
            always_shift,
            disable_shift,
            allowed_moves,
            enable_vpush,
        }
    }

    pub fn physics_settings(&self) -> PhysicsSettings {
        PhysicsSettings {
            mode: self.mode,
            enable_vpush: self.enable_vpush,
        }
    }
}
