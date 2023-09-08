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
    unsafe fn fast_cmp(&self, other: &Self) -> bool {
        unsafe {
            std::mem::transmute::<Self, [u8; 16]>(*self)
                == std::mem::transmute::<Self, [u8; 16]>(*other)
        }
    }

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
    fn update(&mut self, new_outline: Vec<Pointf>) {
        self.outline = new_outline;
        self.init();
    }

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
                .all(|(a, b)| unsafe { a.fast_cmp(b) })
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Hitbox {
    polygon: Polygon,
}

impl PartialEq for Hitbox {
    fn eq(&self, other: &Self) -> bool {
        self.polygon == other.polygon
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

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[pyclass]
enum Move {
    NONE,
    A,
    D,
    W,
    WA,
    WD,
}

impl Move {
    pub fn iterator() -> std::slice::Iter<'static, Move> {
        // static DIRECTIONS: [Move; 3] = [Move::NONE, Move::A, Move::D];
        static DIRECTIONS: [Move; 6] = [Move::NONE, Move::A, Move::D, Move::W, Move::WA, Move::WD];
        DIRECTIONS.iter()
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct PlayerState {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    in_the_air: bool,
}

#[pymethods]
impl PlayerState {
    #[new]
    fn new(x: f64, y: f64, vx: f64, vy: f64, in_the_air: bool) -> Self {
        PlayerState {
            x,
            y,
            vx,
            vy,
            in_the_air,
        }
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
        self.x += TICK_S * self.vx;
        self.y += TICK_S * self.vy;
    }

    fn update_movement(&mut self, mov: Move) {
        self.vx = 0.0;

        if mov == Move::NONE {
            return;
        }
        if mov == Move::A || mov == Move::WA {
            self.vx = -PLAYER_MOVEMENT_SPEED * 1.5;
        }
        if mov == Move::D || mov == Move::WD {
            self.vx = PLAYER_MOVEMENT_SPEED * 1.5;
        }
        if mov == Move::W || mov == Move::WA || mov == Move::WD {
            if !self.in_the_air {
                self.vy = PLAYER_JUMP_SPEED;
                self.in_the_air = true;
            }
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct StaticState {
    objects: Vec<Hitbox>,
}

#[pymethods]
impl StaticState {
    #[new]
    fn new(objects: Vec<Hitbox>) -> Self {
        StaticState { objects }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct PhysState {
    player: PlayerState,
}

#[pymethods]
impl PhysState {
    #[new]
    fn new(player: PlayerState) -> Self {
        PhysState { player }
    }
}

impl PhysState {
    fn tick(&mut self, state: &StaticState) {
        self.player.update_position();
        self.detect_collision(state);
        self.tick_platformer();
    }

    fn detect_collision(&mut self, state: &StaticState) {
        let (list_x, list_y) = self.get_collisions_list(state);

        if list_x.is_empty() && list_y.is_empty() {
            self.player.in_the_air = true;
            return;
        }

        for (o, mpv) in list_x {
            self.align_x_edge(&o, mpv.x);
        }

        let (_, list_y2) = self.get_collisions_list(state);

        for (o, mpv) in list_y2 {
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
    ) -> (Vec<(Hitbox, Pointf)>, Vec<(Hitbox, Pointf)>) {
        let mut collisions_x = Vec::new();
        let mut collisions_y = Vec::new();
        let px = self.player.x;
        let py = self.player.y;

        for o1 in &state.objects {
            if o1.polygon.leftmost_point > px + 16.0 || o1.polygon.rightmost_point < px - 16.0 {
                continue;
            }

            if o1.polygon.lowest_point > py + 16.0 || o1.polygon.highest_point < py - 16.0 {
                continue;
            }

            if let Some(coll) = o1.collides(&self.get_player_hitbox()) {
                let mpv = coll;
                if mpv.x == 0.0 {
                    collisions_y.push((o1.clone(), mpv));
                } else if mpv.y == 0.0 {
                    collisions_x.push((o1.clone(), mpv));
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
        self.player.x == other.player.x
            && self.player.y == other.player.y
            // && self.player.vx == other.player.vx
            && self.player.vy == other.player.vy
    }
}

impl Eq for PhysState {}

impl Hash for PhysState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.player.x.to_le_bytes());
        state.write(&self.player.y.to_le_bytes());
        // state.write(&self.player.vx.to_le_bytes());
        state.write(&self.player.vy.to_le_bytes());
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
    initial_state: PhysState,
    target_state: PhysState,
    static_state: StaticState,
    timeout: u64,
) -> Option<Vec<Move>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<PhysState, (PhysState, Move)> = HashMap::new();
    let mut g_score: HashMap<PhysState, i32> = HashMap::new();
    open_set.push(SearchNode::new(
        heuristic(&target_state.player, &initial_state.player),
        0,
        initial_state.clone(),
    ));
    g_score.insert(initial_state.clone(), 0);

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
            if start.elapsed().unwrap().as_secs() > timeout {
                break;
            }
        }

        for &next_move in Move::iterator() {
            let mut neighbor_state = state.clone();
            neighbor_state.player.update_movement(next_move);
            neighbor_state.tick(&static_state);

            if neighbor_state.close_enough(&target_state) {
                let mut moves = reconstruct_path(&came_from, state);
                moves.push(next_move);
                println!(
                    "found path: iter={:?}; ticks={:?}; elapsed={:?}",
                    iter,
                    ticks + 1,
                    start.elapsed().unwrap().as_secs_f32()
                );
                return Some(moves);
            }

            // println!("[{:?}] before_player: {:?}; after_player: {:?}", next_move, state.player, neighbor_state.player);

            if g_score
                .get(&neighbor_state)
                .is_some_and(|old_ticks| ticks + 1 >= *old_ticks)
            {
                continue;
            }
            let f_score =
                f64::from(ticks + 1) + heuristic(&target_state.player, &neighbor_state.player);
            came_from.insert(neighbor_state.clone(), (state.clone(), next_move));
            g_score.insert(neighbor_state.clone(), ticks + 1);
            open_set.push(SearchNode::new(f_score, ticks + 1, neighbor_state));
        }
    }

    None
}

fn reconstruct_path(
    came_from: &HashMap<PhysState, (PhysState, Move)>,
    current_node: PhysState,
) -> Vec<Move> {
    let mut path = Vec::new();
    let mut current = current_node;

    while let Some((prev, move_direction)) = came_from.get(&current) {
        path.push(*move_direction);
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
    m.add_function(wrap_pyfunction!(astar_search, m)?)?;
    Ok(())
}
