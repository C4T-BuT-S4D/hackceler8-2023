from engine import generics
from engine import hitbox
from engine import modifier


class Ouch(generics.GenericObject):
    def __init__(self, coords, size, min_distance, damage):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + size.width, coords.y),
            hitbox.Point(coords.x + size.width, coords.y - size.height),
            hitbox.Point(coords.x, coords.y - size.height),
        ]
        super().__init__(coords, "Ouch", None, self.perimeter)
        self.modifier = modifier.HealthDamage(min_distance=min_distance, damage=damage)
        self.name = "Ouch"
