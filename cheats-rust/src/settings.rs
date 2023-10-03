use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum GameMode {
    Scroller,
    Platformer,
}

#[pyclass]
#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub mode: GameMode,
    pub timeout: u64,
    pub always_shift: bool,
    pub enable_vpush: bool,
}

#[pymethods]
impl Settings {
    #[new]
    pub fn new(mode: GameMode, timeout: u64, always_shift: bool, enable_vpush: bool) -> Self {
        Settings {
            mode,
            timeout,
            always_shift,
            enable_vpush,
        }
    }
}
