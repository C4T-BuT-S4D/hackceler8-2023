from functools import cache

from engine import generics
from engine import hitbox


class Wall(generics.GenericObject):
    def __init__(self, coords, size, name, perimeter=None):
        self.perimeter = perimeter
        if self.perimeter is None:
            self.perimeter = [
                hitbox.Point(coords.x, coords.y),
                hitbox.Point(coords.x + size.width, coords.y),
                hitbox.Point(coords.x + size.width, coords.y - size.height),
                hitbox.Point(coords.x, coords.y - size.height),
            ]
        super().__init__(coords, "Wall", None, self.perimeter, can_flash=True)
        self.blocking = True
        self.name = name

    @cache
    def get_highest_point(self):
        return super().get_highest_point()

    @cache
    def get_lowest_point(self):
        return super().get_lowest_point()

    @cache
    def get_leftmost_point(self):
        return super().get_leftmost_point()

    @cache
    def get_rightmost_point(self):
        return super().get_rightmost_point()

    @cache
    def get_rect(self):
        return super().get_rect()
