use pyo3::prelude::*;

use env_modifier::EnvModifier;
use geometry::Pointf;
use hitbox::Hitbox;
use moves::{Direction, Move};
use objects::ObjectType;
use physics::{PhysState, PlayerState};
use settings::{GameMode, Settings};

use crate::search::astar_search;
use crate::static_state::StaticState;

mod env_modifier;
mod geometry;
mod hitbox;
mod moves;
mod objects;
mod physics;
mod search;
mod settings;
mod static_state;

#[pyfunction]
fn set_rounded_speeds(vals: Vec<f64>) {
    unsafe {
        physics::ROUNDED_SPEEDS = vals.try_into().unwrap();
    }
}

#[pymodule]
fn cheats_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pointf>()?;
    m.add_class::<Hitbox>()?;
    m.add_class::<PlayerState>()?;
    m.add_class::<StaticState>()?;
    m.add_class::<PhysState>()?;
    m.add_class::<Move>()?;
    m.add_class::<Settings>()?;
    m.add_class::<GameMode>()?;
    m.add_class::<ObjectType>()?;
    m.add_class::<Direction>()?;
    m.add_class::<EnvModifier>()?;
    m.add_function(wrap_pyfunction!(astar_search, m)?)?;
    m.add_function(wrap_pyfunction!(set_rounded_speeds, m)?)?;
    Ok(())
}
