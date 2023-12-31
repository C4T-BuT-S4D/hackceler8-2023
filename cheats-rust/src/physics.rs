use core::hash::Hash;
use std::hash;
use std::hash::Hasher;

use hashbrown::HashMap;
use pyo3::types::PyModule;
use pyo3::{pyclass, pyfunction, pymethods, Python};
use static_init::dynamic;

use crate::settings::SearchSettings;
use crate::{
    env_modifier::EnvModifier,
    geometry::Pointf,
    hitbox::Hitbox,
    moves::{Direction, Move},
    settings::{GameMode, PhysicsSettings},
    static_state::StaticState,
};

pub const TICK_S: f64 = 1.0 / 60.0;
pub const PLAYER_JUMP_SPEED: f64 = 320.0;
pub const PLAYER_MOVEMENT_SPEED: f64 = 160.0;
pub const GRAVITY_CONSTANT: f64 = 6.0;

pub const PUSH_DELTA: f64 = 125.0;
pub const PUSH_SPEED: f64 = 2500.0;

#[dynamic]
static mut ROUND_CACHE: HashMap<HashableF64, f64> = HashMap::new();

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

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct PlayerState {
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,

    #[pyo3(get, set)]
    pub hx_min: f64,
    #[pyo3(get, set)]
    pub hx_max: f64,
    #[pyo3(get, set)]
    pub hy_min: f64,
    #[pyo3(get, set)]
    pub hy_max: f64,

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
        hx_min: f64,
        hx_max: f64,
        hy_min: f64,
        hy_max: f64,
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
            hx_min,
            hx_max,
            hy_min,
            hy_max,
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

impl PlayerState {
    pub fn half_height(&self) -> f64 {
        16.0
    }

    pub fn half_width(&self) -> f64 {
        16.0
    }
}

impl PlayerState {
    pub fn center(&self) -> Pointf {
        Pointf {
            x: self.x,
            y: self.y,
        }
    }

    fn move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
        self.hx_min += dx;
        self.hy_min += dy;
        self.hx_max += dx;
        self.hy_max += dy;
    }

    fn update_position(&mut self) {
        let (dx, dy) = self.round_speeds(self.vx, self.vy);
        self.move_by(dx, dy);
    }

    fn round_speeds(&self, x: f64, y: f64) -> (f64, f64) {
        let hx = HashableF64(x);
        let hy = HashableF64(y);

        let read = ROUND_CACHE.read();

        let mut dx = read.get(&hx).copied();
        let mut dy = read.get(&hy).copied();

        drop(read);

        if let Some(dx) = dx && let Some(dy) = dy {
            return (dx, dy);
        }

        let mut write = ROUND_CACHE.write();

        if dx.is_none() {
            dx = write.get(&hx).copied();
        }
        if dy.is_none() {
            dy = write.get(&hy).copied();
        }

        let dx_set = dx.is_some();
        let dy_set = dy.is_some();

        if dx_set && dy_set {
            return (dx.unwrap(), dy.unwrap());
        }

        Python::with_gil(|py| {
            let builtins = PyModule::import(py, "builtins").unwrap();

            if !dx_set {
                dx = Some(
                    builtins
                        .getattr("round")
                        .unwrap()
                        .call1((TICK_S * x, 5))
                        .unwrap()
                        .extract()
                        .unwrap(),
                );
            }

            if !dy_set {
                dy = Some(
                    builtins
                        .getattr("round")
                        .unwrap()
                        .call1((TICK_S * y, 5))
                        .unwrap()
                        .extract()
                        .unwrap(),
                );
            }
        });

        if !dx_set {
            write.insert(hx, dx.unwrap());
        }
        if !dy_set {
            write.insert(hy, dy.unwrap());
        }

        (dx.unwrap(), dy.unwrap())
    }

    fn update_movement(
        &mut self,
        mov: Move,
        search_settings: &SearchSettings,
        settings: &PhysicsSettings,
        shift_pressed: bool,
        active_modifier: Option<&EnvModifier>,
    ) {
        self.vx = 0.0;
        if settings.mode == GameMode::Scroller {
            self.vy = 0.0;
        }

        if self.can_control_movement {
            if mov == Move::D || mov == Move::WD || mov == Move::SD {
                self.change_direction(
                    search_settings,
                    settings,
                    Direction::E,
                    shift_pressed,
                    active_modifier,
                );
            }
            if mov == Move::A || mov == Move::WA || mov == Move::SA {
                self.change_direction(
                    search_settings,
                    settings,
                    Direction::W,
                    shift_pressed,
                    active_modifier,
                );
            }
            if mov == Move::W || mov == Move::WA || mov == Move::WD {
                self.change_direction(
                    search_settings,
                    settings,
                    Direction::N,
                    shift_pressed,
                    active_modifier,
                );
            }
            if mov == Move::S || mov == Move::SA || mov == Move::SD {
                self.change_direction(
                    search_settings,
                    settings,
                    Direction::S,
                    shift_pressed,
                    active_modifier,
                );
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
        search_settings: &SearchSettings,
        settings: &PhysicsSettings,
        direction: Direction,
        sprinting: bool,
        active_modifier: Option<&EnvModifier>,
    ) {
        self.direction = direction;

        match direction {
            Direction::E | Direction::W => {
                let move_modifier = if sprinting {
                    search_settings.speed_multiplier
                } else {
                    1.0
                };
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
                let move_modifier = if sprinting && search_settings.mode == GameMode::Scroller {
                    search_settings.speed_multiplier
                } else {
                    search_settings.jump_multiplier
                };
                self.vy = if settings.mode == GameMode::Scroller {
                    PLAYER_MOVEMENT_SPEED * move_modifier
                } else {
                    self.jump_speed(active_modifier) * move_modifier
                };
                self.in_the_air = true;
            }
            Direction::S => {
                if self.in_the_air && settings.mode == GameMode::Scroller {
                    let move_modifier = if sprinting {
                        search_settings.speed_multiplier
                    } else {
                        1.0
                    };
                    self.vy = -PLAYER_MOVEMENT_SPEED * move_modifier;
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
#[derive(Clone, Copy, Debug)]
pub struct PhysState {
    pub player: PlayerState,
    settings: PhysicsSettings,
    active_modifier: Option<usize>,
}

#[pymethods]
impl PhysState {
    #[new]
    pub fn new(player: PlayerState, settings: PhysicsSettings) -> Self {
        PhysState {
            player,
            settings,
            active_modifier: None,
        }
    }
}
impl PhysState {
    pub fn tick(
        &mut self,
        mov: Move,
        shift_pressed: bool,
        state: &StaticState,
        search_settings: &SearchSettings,
    ) {
        for speed_tile in &state.speed_tiles {
            speed_tile.tick(self);
        }

        self.player.update_movement(
            mov,
            search_settings,
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

        let dx = if mpv > 0.0 {
            o1.polygon.leftmost_point - self.player.half_width() - 1.0 - self.player.x
        } else {
            o1.polygon.rightmost_point + self.player.half_width() + 1.0 - self.player.x
        };

        self.player.move_by(dx, 0.0);
    }

    fn align_y_edge(&mut self, o1: &Hitbox, mpv: f64) {
        self.player.vy = 0.0;

        let dy = if mpv < 0.0 {
            self.player.in_the_air = false;
            o1.polygon.highest_point + self.player.half_height() - self.player.y
        } else {
            o1.polygon.lowest_point - self.player.half_height() - self.player.y
        };

        self.player.move_by(0.0, dy);
    }

    pub fn get_player_hitbox(&self) -> Hitbox {
        let outline = vec![
            Pointf::new(self.player.hx_min, self.player.hy_min),
            Pointf::new(self.player.hx_max, self.player.hy_min),
            Pointf::new(self.player.hx_max, self.player.hy_max),
            Pointf::new(self.player.hx_min, self.player.hy_max),
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
        self.active_modifier.map(|i| &state.environments[i])
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

impl PhysState {
    fn player_x(&self) -> f64 {
        (self.player.x * 10.0).round() / 10.0
    }

    fn player_y(&self) -> f64 {
        (self.player.y * 10.0).round() / 10.0
    }

    fn player_vy(&self) -> f64 {
        (self.player.vy * 1.0).round() / 1.0
    }
}

impl PartialEq for PhysState {
    fn eq(&self, other: &Self) -> bool {
        if self.settings.simple_geometry {
            if self.player_x() != other.player_x() {
                return false;
            }
            if self.player_y() != other.player_y() {
                return false;
            }
        } else {
            if self.player.x != other.player.x {
                return false;
            }
            if self.player.y != other.player.y {
                return false;
            }
        }

        // if self.settings.simple_geometry {
        //     if self.player.half_height().to_le_bytes() != other.player.half_height().to_le_bytes() {
        //         return false;
        //     }
        //     if self.player.half_width().to_le_bytes() != other.player.half_width().to_le_bytes() {
        //         return false;
        //     }
        // } else {
        //     if self.player.hx_min != other.player.hx_min {
        //         return false;
        //     }
        //     if self.player.hx_max != other.player.hx_max {
        //         return false;
        //     }
        //     if self.player.hy_min != other.player.hy_min {
        //         return false;
        //     }
        //     if self.player.hy_max != other.player.hy_max {
        //         return false;
        //     }
        // }

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
            if self.settings.simple_geometry {
                if self.player.vy != other.player.vy {
                    return false;
                }
            } else {
                if self.player_vy() != other.player_vy() {
                    return false;
                }
            }
        }

        true
    }
}

impl Eq for PhysState {}

impl Hash for PhysState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.settings.simple_geometry {
            state.write(&self.player_x().to_le_bytes());
            state.write(&self.player_y().to_le_bytes());
        } else {
            state.write(&self.player.x.to_le_bytes());
            state.write(&self.player.y.to_le_bytes());
        }

        // if self.settings.simple_geometry {
        //     self.player.half_height().to_le_bytes().hash(state);
        //     self.player.half_width().to_le_bytes().hash(state);
        // } else {
        //     state.write(&self.player.hx_min.to_le_bytes());
        //     state.write(&self.player.hx_max.to_le_bytes());
        //     state.write(&self.player.hy_min.to_le_bytes());
        //     state.write(&self.player.hy_max.to_le_bytes());
        // }

        if self.settings.enable_vpush {
            state.write(&self.player.vpush.to_le_bytes());
            self.player.can_control_movement.hash(state);
            (self.player.direction as u8).hash(state);
        }

        if self.settings.mode == GameMode::Platformer {
            if self.settings.simple_geometry {
                state.write(&self.player_vy().to_le_bytes());
            } else {
                state.write(&self.player.vy.to_le_bytes());
            }
        }
    }
}

#[pyfunction]
pub fn get_transition(
    settings: SearchSettings,
    static_state: StaticState,
    mut state: PhysState,
    next_move: Move,
    shift_pressed: bool,
) -> PlayerState {
    state.detect_env_mod(&static_state);
    state.tick(next_move, shift_pressed, &static_state, &settings);
    state.player
}
