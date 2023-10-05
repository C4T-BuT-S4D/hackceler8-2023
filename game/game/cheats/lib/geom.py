from cheats_rust import Hitbox
from cheats_rust import Pointf

from engine.hitbox import Rectangle


def rect_to_rust_hitbox(rect: Rectangle) -> Hitbox:
    return Hitbox(
        outline=[
            Pointf(x=rect.x1(), y=rect.y1()),
            Pointf(x=rect.x2(), y=rect.y1()),
            Pointf(x=rect.x2(), y=rect.y2()),
            Pointf(x=rect.x1(), y=rect.y2()),
        ]
    )
