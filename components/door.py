from components import magic_items
from engine import generics
from engine import hitbox


class Door(generics.GenericObject):
    def __init__(self, coords, size, name, unlocker):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + size.width, coords.y),
            hitbox.Point(coords.x + size.width, coords.y - size.height),
            hitbox.Point(coords.x, coords.y - size.height),
        ]
        super().__init__(coords, "Door", None, self.perimeter)
        self.blocking = True
        self.name = name
        self.unlocker = unlocker

    def passthrough(self, item_list):
        for i in item_list:
            if i.color == self.unlocker:
                return True
        return False
