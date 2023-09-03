import logging

import components.weapon_systems.base as weapon
import components.projectile as projectile
import engine.hitbox as hitbox


class Gun(weapon.Weapon):
    def __init__(self, coords, collectable):
        super().__init__(
            coords=coords,
            name="Gun",
            weapon_type="projectile",
            damage_type="single",
            damage_algo="constant",
            tileset_path="resources/objects/gun.tmx",
            collectable=collectable,
        )

        # min time in ticks between shots. Multiply by FPS to get seconds
        self.cool_down_delay = 10

        self.last_tick = 0

        self.destroyable = False

    def fire(self, tics, direction, origin):
        if tics - self.last_tick >= self.cool_down_delay:
            self.last_tick = tics
            speed_x = 10
            if direction == "W":
                speed_x = -speed_x
            logging.info(
                f"Firing gun from coordinates {self.x, self.y} at tick "
                f"{tics} in direction {direction}"
            )
            return projectile.Projectile(
                coords=hitbox.Point(self.x, self.y),
                speed_x=speed_x,
                speed_y=0,
                origin=origin,
                damage_algo=self.damage_algo,
                damage_type=self.damage_type,
            )
