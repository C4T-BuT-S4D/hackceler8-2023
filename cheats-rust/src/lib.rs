use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::ops::{Div, Mul, Neg, Sub};
use std::time::SystemTime;

use pyo3::prelude::*;

const TICK_S: f64 = 1.0 / 60.0;
const PLAYER_JUMP_SPEED: f64 = 320.0;
const PLAYER_MOVEMENT_SPEED: f64 = 160.0;
const GRAVITY_CONSTANT: f64 = 6.0;

const HEURISTIC_WEIGHT: f64 = 5.0;
const TARGET_PRECISION: f64 = 16.0;

const PUSH_DELTA: f64 = 125.0;
const PUSH_SPEED: f64 = 2500.0;

#[derive(Copy, Clone, Debug)]
#[pyclass]
struct Pointf {
    x: f64,
    y: f64,
}

#[pymethods]
impl Pointf {
    #[new]
    fn new(x: f64, y: f64) -> Self {
        Pointf { x, y }
    }
}

impl Pointf {
    fn unit_vector(&self) -> Pointf {
        *self / self.len()
    }

    fn angle(&self, other: &Pointf) -> f64 {
        let dot = self.unit_vector() * other.unit_vector();
        180.0 - f64::acos(f64::min(f64::max(dot, -1.0), 1.0)) * 180.0 / PI
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn ortho(&self) -> Pointf {
        Pointf {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Sub for Pointf {
    type Output = Pointf;

    fn sub(self, other: Pointf) -> Pointf {
        Pointf {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Pointf {
    type Output = Pointf;

    fn neg(self) -> Pointf {
        Pointf {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<f64> for Pointf {
    type Output = Pointf;

    fn mul(self, rhs: f64) -> Pointf {
        Pointf {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Pointf> for Pointf {
    type Output = f64;

    fn mul(self, rhs: Pointf) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Div<f64> for Pointf {
    type Output = Pointf;

    fn div(self, rhs: f64) -> Pointf {
        Pointf {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Polygon {
    outline: Vec<Pointf>,
    vectors: Vec<Pointf>,
    angles: Vec<f64>,

    center: Pointf,
    highest_point: f64,
    lowest_point: f64,
    rightmost_point: f64,
    leftmost_point: f64,
}

#[pymethods]
impl Polygon {
    #[new]
    fn new(outline: Vec<Pointf>) -> Self {
        let mut polygon = Polygon {
            outline,
            vectors: vec![],
            angles: vec![],

            center: Pointf { x: 0.0, y: 0.0 },
            highest_point: 0.0,
            lowest_point: 0.0,
            rightmost_point: 0.0,
            leftmost_point: 0.0,
        };
        polygon.init();
        polygon
    }
}

impl Polygon {
    fn init(&mut self) {
        self.get_vectors();
        self.get_angles();
        self.is_convex();
        self.cache_points();
    }

    fn get_vectors(&mut self) {
        self.vectors.clear();
        for i in 0..self.outline.len() {
            self.vectors
                .push(self.outline[(i + 1) % self.outline.len()] - self.outline[i]);
        }
    }

    fn get_angles(&mut self) {
        self.angles.clear();
        for i in 0..self.vectors.len() {
            self.angles
                .push(self.vectors[i].angle(&self.vectors[(i + 1) % self.vectors.len()]));
        }
    }

    fn is_convex(&self) {
        for &angle in &self.angles {
            assert!(angle < 180.0);
        }
    }

    fn cache_points(&mut self) {
        self.lowest_point = f64::INFINITY;
        self.highest_point = f64::NEG_INFINITY;
        self.leftmost_point = f64::INFINITY;
        self.rightmost_point = f64::NEG_INFINITY;

        let mut center_x = 0.0;
        let mut center_y = 0.0;

        for point in &self.outline {
            self.lowest_point = f64::min(self.lowest_point, point.y);
            self.highest_point = f64::max(self.highest_point, point.y);
            self.leftmost_point = f64::min(self.leftmost_point, point.x);
            self.rightmost_point = f64::max(self.rightmost_point, point.x);

            center_x += point.x;
            center_y += point.y;
        }
        self.center = Pointf {
            x: center_x / self.outline.len() as f64,
            y: center_y / self.outline.len() as f64,
        };
    }
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        self.outline.len() == other.outline.len()
            && self
                .outline
                .iter()
                .zip(other.outline.iter())
                .all(|(a, b)| a.x == b.x && a.y == b.y)
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Hitbox {
    polygon: Polygon,
}

impl PartialEq for Hitbox {
    fn eq(&self, other: &Self) -> bool {
        return self.polygon == other.polygon;
    }
}

impl Hash for Hitbox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for point in &self.polygon.outline {
            state.write(&point.x.to_le_bytes());
            state.write(&point.y.to_le_bytes());
        }
    }
}

#[pymethods]
impl Hitbox {
    #[new]
    fn new(outline: Vec<Pointf>) -> Self {
        Hitbox {
            polygon: Polygon::new(outline),
        }
    }
}

impl Hitbox {
    fn collides(&self, other: &Hitbox) -> Option<Pointf> {
        let mut min = f64::INFINITY;
        let mut mpv = Pointf { x: 0.0, y: 0.0 };
        let self_axes = self.get_axes();
        let other_axes = other.get_axes();

        for axis in self_axes.iter().chain(other_axes.iter()) {
            let Some(pv) = self.is_separating_axis(&axis, &other.polygon.outline) else {
                return None;
            };
            let norm = pv * pv;
            if norm < min {
                min = norm;
                mpv = pv;
            }
        }

        let d = self.centers_displacement(other);
        if d * mpv > 0.0 {
            mpv = -mpv;
        }

        Some(mpv)
    }

    fn collides_as_rect(&self, other: &Hitbox) -> bool {
        debug_assert!(self.polygon.outline.len() == 4);
        debug_assert!(other.polygon.outline.len() == 4);

        let (sx1, sx2) = (self.polygon.leftmost_point, self.polygon.rightmost_point);
        let (sy1, sy2) = (self.polygon.lowest_point, self.polygon.highest_point);

        let (ox1, ox2) = (other.polygon.leftmost_point, other.polygon.rightmost_point);
        let (oy1, oy2) = (other.polygon.lowest_point, other.polygon.highest_point);

        return sx1 <= ox2 && sx2 >= ox1 && sy1 <= oy2 && sy2 >= oy1;
    }

    fn centers_displacement(&self, other: &Hitbox) -> Pointf {
        other.polygon.center - self.polygon.center
    }

    fn is_separating_axis(&self, axis: &Pointf, other: &[Pointf]) -> Option<Pointf> {
        let mut min1 = f64::INFINITY;
        let mut max1 = f64::NEG_INFINITY;
        let mut min2 = f64::INFINITY;
        let mut max2 = f64::NEG_INFINITY;

        for point in &self.polygon.outline {
            let proj = *point * *axis;
            min1 = f64::min(min1, proj);
            max1 = f64::max(max1, proj);
        }

        for point in other {
            let proj = *point * *axis;
            min2 = f64::min(min2, proj);
            max2 = f64::max(max2, proj);
        }

        if max1 >= min2 && max2 >= min1 {
            let d = f64::min(max2 - min1, max1 - min2);
            let d_over_o_squared = d / (*axis * *axis) + 1e-10;
            Some(*axis * d_over_o_squared)
        } else {
            None
        }
    }

    fn get_axes(&self) -> Vec<Pointf> {
        self.polygon.vectors.iter().map(|v| v.ortho()).collect()
    }
}

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
enum Move {
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
    pub fn iterator(settings: &Settings) -> std::slice::Iter<'static, Move> {
        // static DIRECTIONS: [Move; 2] = [Move::S, Move::SA];
        // DIRECTIONS.iter()

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
            GameMode::Scroller => DIRECTIONS_SCROLLER.iter(),
            GameMode::Platformer => DIRECTIONS_PLATFORMER.iter(),
        }
    }

    pub fn is_only_vertical(&self) -> bool {
        return *self == Move::S || *self == Move::W;
    }
}

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
enum Direction {
    N,
    S,
    E,
    W,
}

#[pyclass]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum GameMode {
    Scroller,
    Platformer,
}

#[pyclass]
#[derive(Debug, Copy, Clone)]
struct Settings {
    mode: GameMode,
    timeout: u64,
    always_shift: bool,
}

#[pymethods]
impl Settings {
    #[new]
    fn new(mode: GameMode, timeout: u64, always_shift: bool) -> Self {
        Settings {
            mode,
            timeout,
            always_shift,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    vpush: f64,
    can_control_movement: bool,
    direction: Direction,
    in_the_air: bool,
    dead: bool,
}

#[pymethods]
impl PlayerState {
    #[new]
    fn new(
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

fn precision_f64(x: f64, decimals: u32) -> f64 {
    if x == 0. || decimals == 0 {
        0.
    } else {
        let shift = decimals as i32 - x.abs().log10().ceil() as i32;
        let shift_factor = 10_f64.powi(shift);

        (x * shift_factor).round() / shift_factor
    }
}

impl PlayerState {
    fn center(&self) -> Pointf {
        Pointf {
            x: self.x,
            y: self.y,
        }
    }

    fn update_position(&mut self) {
        self.x += precision_f64(TICK_S * self.vx, 5);
        self.y += precision_f64(TICK_S * self.vy, 5);
    }

    fn update_movement(&mut self, mov: Move, settings: &Settings, shift_pressed: bool) {
        self.vx = 0.0;
        if settings.mode == GameMode::Scroller {
            self.vy = 0.0;
        }

        if self.can_control_movement {
            if mov == Move::D || mov == Move::WD || mov == Move::SD {
                self.change_direction(settings, Direction::E, shift_pressed);
            }
            if mov == Move::A || mov == Move::WA || mov == Move::SA {
                self.change_direction(settings, Direction::W, shift_pressed);
            }
            if mov == Move::W || mov == Move::WA || mov == Move::WD {
                self.change_direction(settings, Direction::N, shift_pressed);
            }
            if mov == Move::S || mov == Move::SA || mov == Move::SD {
                self.change_direction(settings, Direction::S, shift_pressed);
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

    fn change_direction(&mut self, settings: &Settings, direction: Direction, sprinting: bool) {
        self.direction = direction;

        match direction {
            Direction::E | Direction::W => {
                let move_modifier = if sprinting { 1.5 } else { 1.0 };
                self.vx = if direction == Direction::E {
                    PLAYER_MOVEMENT_SPEED * move_modifier
                } else {
                    -PLAYER_MOVEMENT_SPEED * move_modifier
                }
            }
            Direction::N => {
                if settings.mode == GameMode::Platformer && self.in_the_air {
                    return;
                }
                self.vy = if settings.mode == GameMode::Scroller {
                    PLAYER_MOVEMENT_SPEED
                } else {
                    PLAYER_JUMP_SPEED
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

#[pyclass]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum ObjectType {
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
struct SpeedTile {
    hitbox: Hitbox,
}

impl SpeedTile {
    fn new(hitbox: Hitbox) -> Self {
        Self { hitbox }
    }

    fn tick(&self, state: &mut PhysState) {
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

#[pyclass]
#[derive(Clone)]
struct StaticState {
    objects: Vec<(Hitbox, ObjectType)>,
    speed_tiles: Vec<SpeedTile>,
    deadly: Vec<Hitbox>,
}

#[pymethods]
impl StaticState {
    #[new]
    fn new(objects: Vec<(Hitbox, ObjectType)>) -> Self {
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
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct PhysState {
    player: PlayerState,
    settings: Settings,
}

#[pymethods]
impl PhysState {
    #[new]
    fn new(player: PlayerState, settings: Settings) -> Self {
        PhysState { player, settings }
    }
}

impl PhysState {
    fn tick(&mut self, mov: Move, shift_pressed: bool, state: &StaticState) {
        for speed_tile in &state.speed_tiles {
            speed_tile.tick(self);
        }
        self.player
            .update_movement(mov, &self.settings, shift_pressed);
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
            self.tick_platformer();
        }
    }

    fn detect_collision(&mut self, state: &StaticState) {
        let (list_x, list_y) = self.get_collisions_list(state);

        if list_x.is_empty() && list_y.is_empty() {
            self.player.in_the_air = true;
            return;
        }

        for (o, t, mpv) in list_x {
            self.align_x_edge(&o, mpv.x);
        }

        let (_, list_y2) = self.get_collisions_list(state);

        for (o, t, mpv) in list_y2 {
            self.align_y_edge(&o, mpv.y);
        }
    }

    fn tick_platformer(&mut self) {
        if self.player.in_the_air {
            self.player.vy -= GRAVITY_CONSTANT;
        }
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

    fn get_collisions_list(
        &self,
        state: &StaticState,
    ) -> (
        Vec<(Hitbox, ObjectType, Pointf)>,
        Vec<(Hitbox, ObjectType, Pointf)>,
    ) {
        let mut collisions_x = Vec::new();
        let mut collisions_y = Vec::new();
        let px = self.player.x;
        let py = self.player.y;

        let player_hitbox = &self.get_player_hitbox();

        for (o1, t) in &state.objects {
            if o1.polygon.leftmost_point > px + 16.0 || o1.polygon.rightmost_point < px - 16.0 {
                continue;
            }

            if o1.polygon.lowest_point > py + 16.0 || o1.polygon.highest_point < py - 16.0 {
                continue;
            }

            if let Some(coll) = o1.collides(player_hitbox) {
                let mpv = coll;
                if mpv.x == 0.0 {
                    collisions_y.push((o1.clone(), *t, mpv));
                } else if mpv.y == 0.0 {
                    collisions_x.push((o1.clone(), *t, mpv));
                }
            }
        }

        (collisions_x, collisions_y)
    }

    fn get_player_hitbox(&self) -> Hitbox {
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

    fn close_enough(&self, target_state: &PhysState) -> bool {
        f64::abs(self.player.x - target_state.player.x) <= TARGET_PRECISION
            && f64::abs(self.player.y - target_state.player.y) <= TARGET_PRECISION
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

        if self.player.vpush != other.player.vpush {
            return false;
        }
        if self.player.can_control_movement != other.player.can_control_movement {
            return false;
        }
        if self.player.direction != other.player.direction {
            return false;
        }

        if self.settings.mode == GameMode::Platformer {
            if self.player.vy != other.player.vy {
                return false;
            }
        }
        return true;
    }
}

impl Eq for PhysState {}

impl Hash for PhysState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.player.x.to_le_bytes());
        state.write(&self.player.y.to_le_bytes());

        state.write(&self.player.vpush.to_le_bytes());
        state.write(&(self.player.can_control_movement as u8).to_le_bytes());
        state.write(&(self.player.direction as u8).to_le_bytes());

        if self.settings.mode == GameMode::Platformer {
            state.write(&self.player.vy.to_le_bytes());
        }
    }
}

#[pyclass]
struct SearchNode {
    cost: f64,
    ticks: i32,
    state: PhysState,
}

#[pymethods]
impl SearchNode {
    #[new]
    fn new(cost: f64, ticks: i32, state: PhysState) -> Self {
        SearchNode { cost, ticks, state }
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(target_state: &PlayerState, current_state: &PlayerState) -> f64 {
    // let xticks = (target_state.x - current_state.x).abs() / (PLAYER_MOVEMENT_SPEED * 1.5 * TICK_S);
    // let yticks = (target_state.y - current_state.y).abs() / (PLAYER_JUMP_SPEED * TICK_S);
    // f64::max(xticks, yticks) * HEURISTIC_WEIGHT
    (target_state.center() - current_state.center()).len() * HEURISTIC_WEIGHT
}

#[pyfunction]
fn astar_search(
    settings: Settings,
    initial_state: PhysState,
    target_state: PhysState,
    static_state: StaticState,
) -> Option<Vec<(Move, bool)>> {
    println!("running astar search with settings {settings:?}");

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<PhysState, (PhysState, Move, bool)> = HashMap::new();
    let mut g_score: HashMap<PhysState, i32> = HashMap::new();
    open_set.push(SearchNode::new(
        heuristic(&target_state.player, &initial_state.player),
        0,
        initial_state.clone(),
    ));
    g_score.insert(initial_state.clone(), 0);

    let shift_variants = if settings.always_shift {
        vec![true]
    } else {
        vec![false, true]
    };

    let start = SystemTime::now();
    let mut iter = 0;
    while let Some(SearchNode {
        cost: _,
        ticks,
        state,
    }) = open_set.pop()
    {
        if *g_score.get(&state).unwrap() < ticks {
            continue;
        }

        iter += 1;
        if iter % 10000 == 0 {
            println!("iter: {iter}");
            if start.elapsed().unwrap().as_secs() > settings.timeout {
                break;
            }
            // if iter > 1000 {
            //     break;
            // }
        }

        // let mut next_move = Move::S;
        // let shift_pressed = true;
        // if state.player.vpush == 3250.0 {
        //     next_move = Move::SA;
        // }

        for &next_move in Move::iterator(&settings) {
            for &shift_pressed in shift_variants.iter() {
                if !settings.always_shift && shift_pressed && next_move.is_only_vertical() {
                    continue;
                }

                let mut neighbor_state = state.clone();
                neighbor_state.tick(next_move, shift_pressed, &static_state);
                // println!("[{:?}] before_player: {:?}; after_player: {:?}", next_move, state.player, neighbor_state.player);

                if neighbor_state.player.dead {
                    continue;
                }

                if neighbor_state.close_enough(&target_state) {
                    let mut moves = reconstruct_path(&came_from, state);
                    moves.push((next_move, shift_pressed));
                    println!(
                        "found path: iter={:?}; ticks={:?}; elapsed={:?}",
                        iter,
                        ticks + 1,
                        start.elapsed().unwrap().as_secs_f32()
                    );
                    return Some(moves);
                }

                // if neighbor_state.player.vpush > 2000.0 {
                // println!("[{next_move:?}] good state: {neighbor_state:?}");
                // }

                if g_score
                    .get(&neighbor_state)
                    .is_some_and(|old_ticks| ticks + 1 >= *old_ticks)
                {
                    continue;
                }
                let f_score =
                    f64::from(ticks + 1) + heuristic(&target_state.player, &neighbor_state.player);
                came_from.insert(
                    neighbor_state.clone(),
                    (state.clone(), next_move, shift_pressed),
                );
                g_score.insert(neighbor_state.clone(), ticks + 1);
                open_set.push(SearchNode::new(f_score, ticks + 1, neighbor_state));
            }
        }
    }

    None
}

fn reconstruct_path(
    came_from: &HashMap<PhysState, (PhysState, Move, bool)>,
    current_node: PhysState,
) -> Vec<(Move, bool)> {
    let mut path = Vec::new();
    let mut current = current_node;

    while let Some((prev, move_direction, shift_pressed)) = came_from.get(&current) {
        path.push((*move_direction, *shift_pressed));
        current = prev.clone();
    }

    path.reverse();
    path
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
    m.add_function(wrap_pyfunction!(astar_search, m)?)?;
    Ok(())
}
