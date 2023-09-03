from engine import generics
from engine import modifier


class Fire(generics.GenericObject):
    def __init__(self, coords, min_distance):
        super().__init__(coords, "Fire", "resources/objects/fire.tmx")
        self.modifier = modifier.HealthDamage(min_distance=min_distance, damage=0.5)
        self.sprite.set_animation("idle")
        self.name = "Fire"
