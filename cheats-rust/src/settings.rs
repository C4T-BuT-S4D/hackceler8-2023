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
    pub simple_geometry: bool,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SearchSettings {
    pub mode: GameMode,
    pub timeout: u64,
    pub always_shift: bool,
    pub disable_shift: bool,
    pub allowed_moves: Vec<Move>,
    pub heuristic_weight: f64,
    pub enable_vpush: bool,
    pub simple_geometry: bool,
    pub state_batch_size: usize,
}

#[pymethods]
impl SearchSettings {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: GameMode,
        timeout: u64,
        always_shift: bool,
        disable_shift: bool,
        allowed_moves: Vec<Move>,
        heuristic_weight: f64,
        enable_vpush: bool,
        simple_geometry: bool,
        state_batch_size: usize,
    ) -> Self {
        Self {
            mode,
            timeout,
            always_shift,
            disable_shift,
            allowed_moves,
            heuristic_weight,
            enable_vpush,
            simple_geometry,
            state_batch_size,
        }
    }

    pub fn physics_settings(&self) -> PhysicsSettings {
        PhysicsSettings {
            mode: self.mode,
            enable_vpush: self.enable_vpush,
            simple_geometry: self.simple_geometry,
        }
    }
}
