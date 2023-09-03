from engine import generics
from engine import hitbox


class MovingPlatform(generics.GenericObject):
    def __init__(self, coords, size, name, perimeter=None):
        self.perimeter = perimeter
        if self.perimeter is None:
            self.perimeter = [
                hitbox.Point(coords.x, coords.y),
                hitbox.Point(coords.x + size.width, coords.y),
                hitbox.Point(coords.x + size.width, coords.y - size.height),
                hitbox.Point(coords.x, coords.y - size.height),
            ]
        super().__init__(coords, "MovingPlatform", None, self.perimeter, can_flash=True)

        self.max_x = coords.x + 20
        self.min_x = coords.x - 20

        self.max_y = coords.y + 20
        self.min_y = coords.y - 20

        self.y_speed = 0.5
        self.x_speed = 0

    def move_around(self):
        if self.y >= self.max_y:
            self.y_speed = -self.y_speed
        if self.y <= self.min_y:
            self.y_speed = -self.y_speed

        self.move(self.x_speed, self.y_speed)
