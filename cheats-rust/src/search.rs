use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    time::SystemTime,
};

use pyo3::{pyclass, pyfunction, pymethods};

use crate::{
    moves::Move,
    physics::{PhysState, PlayerState},
    settings::Settings,
    StaticState,
};

const HEURISTIC_WEIGHT: f64 = 5.0;
const TARGET_PRECISION: f64 = 16.0;

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
pub fn astar_search(
    settings: Settings,
    mut initial_state: PhysState,
    target_state: PhysState,
    static_state: StaticState,
) -> Option<Vec<(Move, bool, PlayerState)>> {
    println!("running astar search with settings {settings:?}");

    initial_state.detect_env_mod(&static_state);

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<PhysState, (PhysState, Move, bool)> = HashMap::new();
    let mut g_score: HashMap<PhysState, i32> = HashMap::new();
    open_set.push(SearchNode::new(
        heuristic(&target_state.player, &initial_state.player),
        0,
        initial_state.clone(),
    ));
    g_score.insert(initial_state, 0);

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

                if neighbor_state.close_enough(&target_state, TARGET_PRECISION) {
                    let mut moves = reconstruct_path(&came_from, state);
                    moves.push((next_move, shift_pressed, neighbor_state.player));
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
) -> Vec<(Move, bool, PlayerState)> {
    let mut path = Vec::new();
    let mut current = current_node;

    while let Some((prev, move_direction, shift_pressed)) = came_from.get(&current) {
        path.push((*move_direction, *shift_pressed, current.player));
        current = prev.clone();
    }

    path.reverse();
    path
}
