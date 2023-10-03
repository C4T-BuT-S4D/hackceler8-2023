use pyo3::{pyclass, pymethods};

use crate::hitbox::Hitbox;

#[pyclass]
#[derive(Debug, Clone)]
pub struct EnvModifier {
    pub hitbox: Hitbox,
    pub jump_speed: f64,
    pub jump_height: f64,
    pub walk_speed: f64,
    pub run_speed: f64,
    pub gravity: f64,
    pub jump_override: bool,
}

#[pymethods]
impl EnvModifier {
    #[new]
    pub fn new(
        hitbox: Hitbox,
        jump_speed: f64,
        jump_height: f64,
        walk_speed: f64,
        run_speed: f64,
        gravity: f64,
        jump_override: bool,
    ) -> Self {
        EnvModifier {
            hitbox,
            jump_speed,
            jump_height,
            walk_speed,
            run_speed,
            gravity,
            jump_override,
        }
    }
}
