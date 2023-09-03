from engine import generics
from engine import hitbox


class ExitArea(generics.GenericObject):
    def __init__(self, coords, size, name):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + size.width, coords.y),
            hitbox.Point(coords.x + size.width, coords.y - size.height),
            hitbox.Point(coords.x, coords.y - size.height),
        ]
        super().__init__(coords, "ExitArea", None, self.perimeter)
        self.blocking = True
        self.name = name
