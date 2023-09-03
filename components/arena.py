from engine import generics
from engine import hitbox


class Arena(generics.GenericObject):
    def __init__(self, coords, size, name):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + size.width, coords.y),
            hitbox.Point(coords.x + size.width, coords.y - size.height),
            hitbox.Point(coords.x, coords.y - size.height),
        ]
        super().__init__(
            coords, nametype="Arena", tileset_path=None, outline=self.perimeter
        )
        self.blocking = True
        self.name = name
