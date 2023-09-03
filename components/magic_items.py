from enum import Enum

from engine import generics
from engine import hitbox


def _get_tileset(name):
    if name.startswith("key_"):
        return "resources/objects/items/key_%s.tmx" % name.split("_")[1]
    return None


class ItemType(Enum):
    TYPE_PURPLE = "purple"


class Item(generics.GenericObject):
    def __init__(self, coords, name, display_name, color):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + 10, coords.y),
            hitbox.Point(coords.x + 10, coords.y - 10),
            hitbox.Point(coords.x, coords.y - 10),
        ]
        super().__init__(coords, "Item", _get_tileset(name), self.perimeter)
        self.blocking = True
        self.name = name
        self.display_name = display_name
        self.color = color
        if self.sprite is not None:
            w, h = self.sprite.get_dimensions()
            self._update(
                [
                    hitbox.Point(coords.x - w / 2, coords.y - h / 2),
                    hitbox.Point(coords.x + w / 2, coords.y - h / 2),
                    hitbox.Point(coords.x + w / 2, coords.y + h / 2),
                    hitbox.Point(coords.x - w / 2, coords.y + h / 2),
                ]
            )
