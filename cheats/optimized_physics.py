import logging
from dataclasses import dataclass

import constants
from engine import generics, physics, hitbox


@dataclass
class PlayerState:
    in_the_air: bool
    x: int
    y: int
    x_speed: int
    y_speed: int

    def place_at(self, x: int, y: int):
        self.x = x
        self.y = y

    def update_position(self):
        self.x += constants.TICK_S * self.x_speed
        self.y += constants.TICK_S * self.y_speed

    @property
    def hitbox(self):
        return hitbox.Hitbox(
            [
                hitbox.Point(self.x - 16, self.y - 16),
                hitbox.Point(self.x + 16, self.y - 16),
                hitbox.Point(self.x + 16, self.y + 16),
                hitbox.Point(self.x - 16, self.y + 16),
            ]
        )

    def update_movement(self, pressed_keys: set[str]):
        self.x_speed = 0

        running_mod = 1.5
        if "D" in pressed_keys and "A" not in pressed_keys:
            self.x_speed = physics.PLAYER_MOVEMENT_SPEED * running_mod
        if "A" in pressed_keys and "D" not in pressed_keys:
            self.x_speed = -physics.PLAYER_MOVEMENT_SPEED * running_mod
        if "W" in pressed_keys and not self.in_the_air:
            self.y_speed = physics.PLAYER_JUMP_SPEED
            self.in_the_air = True


class OptimizedPhysicsEngine:
    def __init__(
        self,
        player: PlayerState,
        static_objects: list[generics.GenericObject],
        gravity: int = physics.GRAVITY_CONSTANT,
    ):
        self.og_gravity = gravity
        self.og_jump_speed = physics.PLAYER_JUMP_SPEED
        self.og_movement_speed = physics.PLAYER_MOVEMENT_SPEED
        self.og_run_speed = physics.PLAYER_RUN_SPEED

        self.gravity = None
        self.jump_speed = None
        self.movement_speed = None
        self.run_speed = None

        self.jump_override = False

        self.original_params = [
            self.og_gravity,
            self.og_jump_speed,
            self.og_movement_speed,
            self.og_run_speed,
        ]

        self.static_objects = static_objects
        self.player = player

        self._reset_parameters()

    def _reset_parameters(self):
        (
            self.gravity,
            self.jump_speed,
            self.movement_speed,
            self.run_speed,
        ) = self.original_params
        self.jump_override = False
        self.player.jump_override = self.jump_override
        self.current_env = "generic"

    def tick(self):
        self.player.update_position()
        self._detect_collision()
        self._tick_platformer()

    def _tick_platformer(self):
        if self.player.in_the_air:
            logging.debug("updating gravity speed for player")

            self.player.y_speed -= self.gravity

    def _get_collisions_list(self):
        collisions_x, collisions_y = [], []
        px, py = self.player.x, self.player.y

        for o1 in self.static_objects:
            if o1.get_leftmost_point() > px + 16 or o1.get_rightmost_point() < px - 16:
                continue
            if o1.get_lowest_point() > py + 16 or o1.get_highest_point() < py - 16:
                continue

            c, mpv = o1.collides(self.player.hitbox)
            if c:
                if mpv[0] == 0.0:
                    collisions_y.append((o1, mpv))
                elif mpv[1] == 0.0:
                    collisions_x.append((o1, mpv))
        return collisions_x, collisions_y

    def _detect_collision(self):
        collisions_x, collisions_y = self._get_collisions_list()
        if len(collisions_x) + len(collisions_y) == 0:
            self.player.in_the_air = True
            return

        for o, mpv in collisions_x:
            self._align_x_edge(o, mpv[0])

        _, collisions_y = self._get_collisions_list()
        for o, mpv in collisions_y:
            self._align_y_edge(o, mpv[1])

    def _align_y_edge(self, o1, mpv):
        self.player.y_speed = 0
        if mpv < 0:
            max_y_o2 = o1.get_highest_point()
            self.player.place_at(self.player.x, max_y_o2 + 16)
            self.player.in_the_air = False

        else:
            min_y_o2 = o1.get_lowest_point()
            self.player.place_at(self.player.x, min_y_o2 - 16)

    def _align_x_edge(self, o1, mpv):
        self.player.x_speed = 0
        self.player.in_the_air = True

        if mpv > 0:
            min_x_o2 = o1.get_leftmost_point()
            self.player.x_speed = 0
            self.player.place_at(min_x_o2 - 17, self.player.y)
        else:
            max_x_o2 = o1.get_rightmost_point()
            self.player.x_speed = 0
            self.player.place_at(max_x_o2 + 17, self.player.y)
