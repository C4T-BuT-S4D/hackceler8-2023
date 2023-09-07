import math
import time
from dataclasses import dataclass, field
from copy import deepcopy

from sortedcontainers import SortedKeyList

from cheats import optimized_physics
from engine import generics


@dataclass
class State:
    ticks: int
    player_state: optimized_physics.PlayerState
    tick_keys: list[set[str]] = field(default_factory=list)


class PathFinder:
    static_objs: list[generics.GenericObject]
    states: SortedKeyList[State]
    dist: dict[(int, int), State]

    def __init__(self, initial_state: State, static_objs: list[generics.GenericObject]):
        self.static_objs = static_objs
        self.states = SortedKeyList([], key=lambda x: x[0])
        self.dist = {}

        initial_state.ticks = 0
        self.add_state((-1000, -1000), initial_state)

    def search(self, x: int, y: int):
        target = None
        iter = 0
        start = time.time()
        while len(self.states) > 0:
            iter += 1
            if iter % 1000 == 0:
                print("iter: ", iter)
                if time.time() - start > 10:
                    print('search timed out')
                    break

            _, state = self.states.pop(0)

            sx, sy = state.player_state.x, state.player_state.y
            if (sx, sy) in self.dist and self.dist[sx, sy].ticks < state.ticks:
                continue

            # print("state: ", state, "target: ", x, y)

            found = Falsepy
            for move in ["A", "D", "WA", "WD", "W", ""]:
                if (res := self.process((x, y), state, set(move))) is not None:
                    found = True
                    target = res
                    break
            if found:
                break

        if target is not None:
            print(f"Requested {x, y}, found {target} in {self.dist.get(target)} ticks")
            return self.dist.get(target)
        else:
            print("Path not found")

    def process(self, need: tuple[int, int], state: State, keys: set[str]):
        new_player = deepcopy(state.player_state)

        ticks_to_emulate = 1
        start = time.time()
        # print(f'[{keys}] before:', state)
        engine = optimized_physics.OptimizedPhysicsEngine(new_player, self.static_objs)
        for t in range(ticks_to_emulate):
            new_player.update_movement(keys)
            engine.tick()
        # print(f'[{keys}] after:', state)
        print("emulation time: ", time.time() - start)

        return self.add_state(
            need,
            State(
                ticks=state.ticks + ticks_to_emulate,
                player_state=new_player,
                tick_keys=state.tick_keys + [keys] * ticks_to_emulate,
            ),
        )

    def add_state(self, need: tuple[int, int], state: State):
        x, y = state.player_state.x, state.player_state.y
        if (x, y) in self.dist and self.dist[x, y].ticks <= state.ticks:
            return None

        metric = math.sqrt((need[0] - x) ** 2 + (need[1] - y) ** 2)
        # metric = abs(need[0] - x) + abs(need[1] - y)
        metric *= 10
        # metric = 0
        self.states.add((state.ticks + metric, state))
        self.dist[x, y] = state

        if abs(need[0] - x) <= 16 and abs(need[1] - y) <= 16:
            return x, y
        # if state.ticks > 500:
        #     return -1, -1
