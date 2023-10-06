use std::{cmp::Ordering, collections::BinaryHeap, sync::atomic::AtomicUsize, time::SystemTime};

use hashbrown::HashMap;
use pyo3::{pyclass, pyfunction, pymethods, Python};
use rayon::prelude::*;

use crate::static_state::StaticState;
use crate::{
    moves::Move,
    physics::{PhysState, PlayerState, PLAYER_JUMP_SPEED, PLAYER_MOVEMENT_SPEED, TICK_S},
    settings::{GameMode, SearchSettings},
};

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

fn heuristic(
    settings: &SearchSettings,
    target_state: &PlayerState,
    current_state: &PlayerState,
) -> f64 {
    let y_speed = if settings.mode == GameMode::Platformer {
        PLAYER_JUMP_SPEED
    } else {
        PLAYER_MOVEMENT_SPEED
    };

    let xticks = (target_state.x - current_state.x).abs() / (PLAYER_MOVEMENT_SPEED * 1.5 * TICK_S);
    let yticks = (target_state.y - current_state.y).abs() / (y_speed * TICK_S);
    f64::max(xticks, yticks) * settings.heuristic_weight
    // (target_state.center() - current_state.center()).len() * settings.heuristic_weight
    //0.0
}

struct SearchResult<'a> {
    prev_state: &'a PhysState,
    ticks: i32,
    f_score: f64,
    next_move: Move,
    shift: bool,
    next_state: PhysState,
}

#[pyfunction]
pub fn astar_search(
    py: Python<'_>,
    settings: SearchSettings,
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
        heuristic(&settings, &target_state.player, &initial_state.player),
        0,
        initial_state,
    ));
    g_score.insert(initial_state, 0);

    let shift_variants = if settings.always_shift {
        vec![true]
    } else if settings.disable_shift {
        vec![false]
    } else {
        vec![false, true]
    };

    let allowed_moves = Move::all(&settings);

    let start = SystemTime::now();
    let iter = AtomicUsize::new(0);

    let mut to_process = Vec::new();

    py.allow_threads(|| {
        loop {
            if start.elapsed().unwrap().as_secs() > settings.timeout {
                break;
            }

            to_process.drain(..);
            while to_process.len() < settings.state_batch_size && let Some(SearchNode {
                                                                               cost: _,
                                                                               ticks,
                                                                               state,
                                                                           }) = open_set.pop()
            {
                to_process.push((ticks, state));
            }

            if to_process.is_empty() {
                break;
            }

            println!(
                "processed {iter:?} iterations, to_process: {:} elems",
                to_process.len()
            );

            let next_states: Vec<_> = to_process
                .par_iter()
                .filter_map(|(ticks, state)| {
                    if *g_score.get(state).unwrap() < *ticks {
                        return None;
                    }

                    iter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let mut next_states = Vec::new();

                    for &next_move in &allowed_moves {
                        for &shift_pressed in shift_variants.iter() {
                            if !settings.always_shift
                                && shift_pressed
                                && next_move.is_only_vertical()
                            {
                                continue;
                            }

                            let mut neighbor_state = *state;
                            neighbor_state.tick(next_move, shift_pressed, &static_state, &settings);

                            if neighbor_state.player.dead {
                                continue;
                            }

                            if g_score
                                .get(&neighbor_state)
                                .is_some_and(|old_ticks| ticks + 1 >= *old_ticks)
                            {
                                continue;
                            }

                            let f_score = f64::from(ticks + 1)
                                + heuristic(
                                    &settings,
                                    &target_state.player,
                                    &neighbor_state.player,
                                );

                            next_states.push(SearchResult {
                                prev_state: state,
                                ticks: *ticks,
                                f_score,
                                next_move,
                                shift: shift_pressed,
                                next_state: neighbor_state,
                            });
                        }
                    }

                    Some(next_states)
                })
                .collect();

            for search_results in next_states {
                for sr in search_results {
                    if sr.next_state.close_enough(&target_state, TARGET_PRECISION) {
                        let mut moves = reconstruct_path(&came_from, *sr.prev_state);
                        moves.push((sr.next_move, sr.shift, sr.next_state.player));
                        println!(
                            "found path: iter={:?}; ticks={:?}; elapsed={:?}",
                            iter,
                            sr.ticks + 1,
                            start.elapsed().unwrap().as_secs_f32()
                        );
                        return Some(moves);
                    }

                    if g_score
                        .get(&sr.next_state)
                        .is_some_and(|old_ticks| sr.ticks + 1 >= *old_ticks)
                    {
                        continue;
                    }

                    came_from.insert(sr.next_state, (*sr.prev_state, sr.next_move, sr.shift));
                    g_score.insert(sr.next_state, sr.ticks + 1);
                    open_set.push(SearchNode::new(sr.f_score, sr.ticks + 1, sr.next_state));
                }
            }
        }

        None
    })
}

fn reconstruct_path(
    came_from: &HashMap<PhysState, (PhysState, Move, bool)>,
    current_node: PhysState,
) -> Vec<(Move, bool, PlayerState)> {
    let mut path = Vec::new();
    let mut current = current_node;

    while let Some((prev, move_direction, shift_pressed)) = came_from.get(&current) {
        path.push((*move_direction, *shift_pressed, current.player));
        current = *prev;
    }

    path.reverse();
    path
}
