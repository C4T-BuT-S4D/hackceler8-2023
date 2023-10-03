use pyo3::{pyclass, pymethods};

use crate::{
    env_modifier::EnvModifier,
    hitbox::Hitbox,
    objects::{ObjectType, SpeedTile},
};

#[pyclass]
#[derive(Clone)]
pub struct StaticState {
    pub objects: Vec<(Hitbox, ObjectType)>,
    pub speed_tiles: Vec<SpeedTile>,
    pub deadly: Vec<Hitbox>,
    pub environments: Vec<EnvModifier>,
}

#[pymethods]
impl StaticState {
    #[new]
    pub fn new(objects: Vec<(Hitbox, ObjectType)>, environments: Vec<EnvModifier>) -> Self {
        let mut speed_tiles = Vec::new();
        let mut deadly = Vec::new();
        let mut other_objects = Vec::new();
        for (hitbox, t) in objects {
            match t {
                ObjectType::SpeedTile => {
                    speed_tiles.push(SpeedTile::new(hitbox));
                }
                ObjectType::Spike => {
                    deadly.push(hitbox);
                }
                _ => {
                    other_objects.push((hitbox, t));
                }
            }
        }
        StaticState {
            objects: other_objects,
            speed_tiles,
            deadly,
            environments,
        }
    }
}
