from engine import generics
from engine import modifier


class Fountain(generics.GenericObject):
    def __init__(self, coords, min_distance):
        super().__init__(coords, "Fountain", "resources/objects/fire.tmx")
        self.modifier = modifier.HealthIncreaser(min_distance=min_distance, benefit=0.5)
        self.sprite.set_animation("idle")
        self.name = "Fire"
