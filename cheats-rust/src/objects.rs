use pyo3::pyclass;

use crate::{
    hitbox::Hitbox,
    moves::Direction,
    physics::{PhysState, PUSH_SPEED},
};

#[pyclass]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ObjectType {
    Wall,
    Spike,
    SpeedTile,
}

impl ObjectType {
    pub fn is_deadly(&self) -> bool {
        *self == Self::Spike
    }
}

#[derive(Debug, Clone)]
pub struct SpeedTile {
    hitbox: Hitbox,
}

impl SpeedTile {
    pub fn new(hitbox: Hitbox) -> Self {
        Self { hitbox }
    }

    pub fn tick(&self, state: &mut PhysState) {
        let already_pushing = state.player.can_control_movement && state.player.vpush > 0.0;
        if self.hitbox.collides(&state.get_player_hitbox()).is_some() {
            state.player.can_control_movement = false;
            state.player.direction = Direction::N;
            if !already_pushing {
                state.player.vpush += PUSH_SPEED;
            }
        } else {
            state.player.can_control_movement = true;
        }
    }
}
