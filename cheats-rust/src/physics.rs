use core::hash::Hash;
use std::hash::Hasher;

use pyo3::types::PyModule;
use pyo3::{pyclass, pyfunction, pymethods, Python};
use static_init::dynamic;
use std::collections::HashMap;
use std::hash;

use crate::{
    env_modifier::EnvModifier,
    geometry::Pointf,
    hitbox::Hitbox,
    moves::{Direction, Move},
    settings::{GameMode, Settings},
    static_state::StaticState,
};

pub const TICK_S: f64 = 1.0 / 60.0;
pub const PLAYER_JUMP_SPEED: f64 = 320.0;
pub const PLAYER_MOVEMENT_SPEED: f64 = 160.0;
pub const GRAVITY_CONSTANT: f64 = 6.0;

pub const PUSH_DELTA: f64 = 125.0;
pub const PUSH_SPEED: f64 = 2500.0;

#[pyclass]
#[derive(Clone, Debug)]
pub struct PlayerState {
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,
    #[pyo3(get, set)]
    pub vx: f64,
    #[pyo3(get, set)]
    pub vy: f64,
    #[pyo3(get, set)]
    pub vpush: f64,
    #[pyo3(get, set)]
    pub can_control_movement: bool,
    #[pyo3(get, set)]
    pub direction: Direction,
    #[pyo3(get, set)]
    pub in_the_air: bool,
    #[pyo3(get, set)]
    pub dead: bool,
}

#[pymethods]
impl PlayerState {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: f64,
        y: f64,
        vx: f64,
        vy: f64,
        vpush: f64,
        can_control_movement: bool,
        direction: Direction,
        in_the_air: bool,
    ) -> Self {
        PlayerState {
            x,
            y,
            vx,
            vy,
            vpush,
            can_control_movement,
            direction,
            in_the_air,
            dead: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct HashableF64(f64);

impl HashableF64 {
    fn key(&self) -> u64 {
        self.0.to_bits()
    }
}

impl hash::Hash for HashableF64 {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.key().hash(state)
    }
}

impl PartialEq for HashableF64 {
    fn eq(&self, other: &HashableF64) -> bool {
        self.key() == other.key()
    }
}

impl Eq for HashableF64 {}

#[dynamic(0)]
static mut ROUND_CACHE: HashMap<HashableF64, HashableF64> = HashMap::new();

fn round_speed(x: f64) -> f64 {
    unsafe {
        (ROUND_CACHE)
            .entry(HashableF64(x))
            .or_insert_with(|| {
                HashableF64(Python::with_gil(|py| {
                    let builtins = PyModule::import(py, "builtins").unwrap();
                    let ret: f64 = builtins
                        .getattr("round")
                        .unwrap()
                        .call1((TICK_S * x, 5))
                        .unwrap()
                        .extract()
                        .unwrap();
                    ret
                }))
            })
            .0
    }
}

impl PlayerState {
    pub fn center(&self) -> Pointf {
        Pointf {
            x: self.x,
            y: self.y,
        }
    }

    fn update_position(&mut self) {
        //self.x += precision_f64(TICK_S * self.vx, 5);
        //self.y += precision_f64(TICK_S * self.vy, 5);
        self.x += round_speed(self.vx);
        self.y += round_speed(self.vy);
    }

    fn update_movement(
        &mut self,
        mov: Move,
        settings: &Settings,
        shift_pressed: bool,
        active_modifier: Option<&EnvModifier>,
    ) {
        self.vx = 0.0;
        if settings.mode == GameMode::Scroller {
            self.vy = 0.0;
        }

        if self.can_control_movement {
            if mov == Move::D || mov == Move::WD || mov == Move::SD {
                self.change_direction(settings, Direction::E, shift_pressed, active_modifier);
            }
            if mov == Move::A || mov == Move::WA || mov == Move::SA {
                self.change_direction(settings, Direction::W, shift_pressed, active_modifier);
            }
            if mov == Move::W || mov == Move::WA || mov == Move::WD {
                self.change_direction(settings, Direction::N, shift_pressed, active_modifier);
            }
            if mov == Move::S || mov == Move::SA || mov == Move::SD {
                self.change_direction(settings, Direction::S, shift_pressed, active_modifier);
            }

            self.vpush = f64::max(0.0, self.vpush - PUSH_DELTA);
        }

        if self.vpush > 0.0 {
            match self.direction {
                Direction::N => {
                    self.vy += self.vpush;
                }
                Direction::S => {
                    self.vy -= self.vpush;
                }
                Direction::E => {
                    self.vx += self.vpush;
                }
                Direction::W => {
                    self.vx -= self.vpush;
                }
            }
        }
    }

    fn change_direction(
        &mut self,
        settings: &Settings,
        direction: Direction,
        sprinting: bool,
        active_modifier: Option<&EnvModifier>,
    ) {
        self.direction = direction;

        match direction {
            Direction::E | Direction::W => {
                let move_modifier = if sprinting { 1.5 } else { 1.0 };
                self.vx = if direction == Direction::E {
                    self.movement_speed(active_modifier) * move_modifier
                } else {
                    -self.movement_speed(active_modifier) * move_modifier
                }
            }
            Direction::N => {
                if settings.mode == GameMode::Platformer
                    && self.in_the_air
                    && !self.jump_override(active_modifier)
                {
                    return;
                }
                self.vy = if settings.mode == GameMode::Scroller {
                    PLAYER_MOVEMENT_SPEED
                } else {
                    self.jump_speed(active_modifier)
                };
                self.in_the_air = true;
            }
            Direction::S => {
                if self.in_the_air && settings.mode == GameMode::Scroller {
                    self.vy = -PLAYER_MOVEMENT_SPEED;
                }
            }
        }
    }
}

impl PlayerState {
    fn jump_speed(&self, active_modifier: Option<&EnvModifier>) -> f64 {
        PLAYER_JUMP_SPEED * active_modifier.map_or(1.0, |modifier| modifier.jump_speed)
    }

    fn movement_speed(&self, active_modifier: Option<&EnvModifier>) -> f64 {
        PLAYER_MOVEMENT_SPEED * active_modifier.map_or(1.0, |modifier| modifier.walk_speed)
    }

    fn jump_override(&self, active_modifier: Option<&EnvModifier>) -> bool {
        active_modifier.map_or(false, |modifier| modifier.jump_override)
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PhysState {
    pub player: PlayerState,
    settings: Settings,
    active_modifier: Option<usize>,
}

#[pymethods]
impl PhysState {
    #[new]
    pub fn new(player: PlayerState, settings: Settings) -> Self {
        PhysState {
            player,
            settings,
            active_modifier: None,
        }
    }
}

impl PhysState {
    pub fn tick(&mut self, mov: Move, shift_pressed: bool, state: &StaticState) {
        for speed_tile in &state.speed_tiles {
            speed_tile.tick(self);
        }

        self.player.update_movement(
            mov,
            &self.settings,
            shift_pressed,
            self.get_active_modifier(state),
        );
        self.player.update_position();

        let player_hitbox = self.get_player_hitbox();
        for obj in &state.deadly {
            if player_hitbox.collides_as_rect(obj) {
                self.player.dead = true;
                return;
            }
        }
        self.detect_collision(state);
        if self.settings.mode == GameMode::Platformer {
            self.tick_platformer(state);
        }
    }

    fn detect_collision(&mut self, state: &StaticState) {
        let (list_x, list_y) = self.get_collisions_list(state);

        if list_x.is_empty() && list_y.is_empty() {
            self.player.in_the_air = true;
            return;
        }

        for collision in list_x {
            self.align_x_edge(&collision.hitbox, collision.mpv.x);
        }

        let (_, list_y2) = self.get_collisions_list(state);

        for collision in list_y2 {
            self.align_y_edge(&collision.hitbox, collision.mpv.y);
        }
    }

    fn tick_platformer(&mut self, state: &StaticState) {
        if self.settings.mode == GameMode::Platformer {
            self.detect_env_mod(state);
        }
        if self.player.in_the_air {
            self.player.vy -= self.gravity(self.get_active_modifier(state));
        }
    }

    pub fn detect_env_mod(&mut self, state: &StaticState) {
        let player_hitbox = self.get_player_hitbox();
        for (i, env) in state.environments.iter().enumerate() {
            if env.hitbox.collides(&player_hitbox).is_some() {
                self.active_modifier = Some(i);
                return;
            }
        }

        self.active_modifier = None;
    }

    fn align_x_edge(&mut self, o1: &Hitbox, mpv: f64) {
        self.player.vx = 0.0;
        self.player.in_the_air = true;

        if mpv > 0.0 {
            self.player.x = o1.polygon.leftmost_point - 17.0;
        } else {
            self.player.x = o1.polygon.rightmost_point + 17.0;
        }
    }

    fn align_y_edge(&mut self, o1: &Hitbox, mpv: f64) {
        self.player.vy = 0.0;

        if mpv < 0.0 {
            self.player.y = o1.polygon.highest_point + 16.0;
            self.player.in_the_air = false;
        } else {
            self.player.y = o1.polygon.lowest_point - 16.0;
        }
    }

    pub fn get_player_hitbox(&self) -> Hitbox {
        let px = self.player.x;
        let py = self.player.y;
        let outline = vec![
            Pointf::new(px - 16.0, py - 16.0),
            Pointf::new(px + 16.0, py - 16.0),
            Pointf::new(px + 16.0, py + 16.0),
            Pointf::new(px - 16.0, py + 16.0),
        ];
        Hitbox::new(outline)
    }

    pub fn close_enough(&self, target_state: &PhysState, precision: f64) -> bool {
        f64::abs(self.player.x - target_state.player.x) <= precision
            && f64::abs(self.player.y - target_state.player.y) <= precision
    }
}

impl PhysState {
    fn get_active_modifier<'a>(&self, state: &'a StaticState) -> Option<&'a EnvModifier> {
        self.active_modifier
            .and_then(|i| Some(&state.environments[i]))
    }

    fn gravity(&self, active_lifetime: Option<&EnvModifier>) -> f64 {
        GRAVITY_CONSTANT * active_lifetime.map_or(1.0, |modifier| modifier.gravity)
    }
}

struct Collision {
    hitbox: Hitbox,
    mpv: Pointf,
}

impl PhysState {
    fn get_collisions_list(&self, state: &StaticState) -> (Vec<Collision>, Vec<Collision>) {
        let mut collisions_x = Vec::new();
        let mut collisions_y = Vec::new();
        let px = self.player.x;
        let py = self.player.y;

        let player_hitbox = &self.get_player_hitbox();

        for (o1, _) in &state.objects {
            if o1.polygon.leftmost_point > px + 16.0 || o1.polygon.rightmost_point < px - 16.0 {
                continue;
            }

            if o1.polygon.lowest_point > py + 16.0 || o1.polygon.highest_point < py - 16.0 {
                continue;
            }

            if let Some(coll) = o1.collides(player_hitbox) {
                let mpv = coll;
                if mpv.x == 0.0 {
                    collisions_y.push(Collision {
                        hitbox: o1.clone(),
                        mpv,
                    });
                } else if mpv.y == 0.0 {
                    collisions_x.push(Collision {
                        hitbox: o1.clone(),
                        mpv,
                    });
                }
            }
        }

        (collisions_x, collisions_y)
    }
}

impl PartialEq for PhysState {
    fn eq(&self, other: &Self) -> bool {
        if self.player.x != other.player.x {
            return false;
        }
        if self.player.y != other.player.y {
            return false;
        }

        if self.settings.enable_vpush {
            if self.player.vpush != other.player.vpush {
                return false;
            }
            if self.player.can_control_movement != other.player.can_control_movement {
                return false;
            }
            if self.player.direction != other.player.direction {
                return false;
            }
        }

        if self.settings.mode == GameMode::Platformer {
            if self.player.vy != other.player.vy {
                return false;
            }
        }

        true
    }
}

impl Eq for PhysState {}

impl Hash for PhysState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.player.x.to_le_bytes());
        state.write(&self.player.y.to_le_bytes());

        if self.settings.enable_vpush {
            state.write(&self.player.vpush.to_le_bytes());
            state.write(&(self.player.can_control_movement as u8).to_le_bytes());
            state.write(&(self.player.direction as u8).to_le_bytes());
        }

        if self.settings.mode == GameMode::Platformer {
            state.write(&self.player.vy.to_le_bytes());
        }
    }
}

fn precision_f64(x: f64, decimals: u32) -> f64 {
    if x == 0. || decimals == 0 {
        0.
    } else {
        let shift = decimals as i32 - x.abs().log10().ceil() as i32;
        let shift_factor = 10_f64.powi(shift);

        (x * shift_factor).round() / shift_factor
    }
}

#[pyfunction]
pub fn get_transition(
    static_state: StaticState,
    mut state: PhysState,
    next_move: Move,
    shift_pressed: bool,
) -> PlayerState {
    state.tick(next_move, shift_pressed, &static_state);
    state.player
}
