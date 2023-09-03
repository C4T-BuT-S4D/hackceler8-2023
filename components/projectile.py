from engine import generics


class Projectile(generics.GenericObject):
    def __init__(
        self,
        coords,
        speed_x,
        speed_y,
        origin,
        damage_algo="constant",
        damage_type="single",
        base_damage=10,
    ):
        super().__init__(
            coords,
            nametype="Projectile",
            tileset_path="resources/objects/fire.tmx",
            outline=None,
        )
        self.set_speed(speed_x, speed_y)
        self.origin = origin
        self.damage_algo = damage_algo
        self.damage_type = damage_type
        self.base_damage = base_damage

    def check_oob(self):
        if self.x < -10000 or self.x > 10000:
            return True
        if self.y < -10000 or self.y > 10000:
            return True

        return False
