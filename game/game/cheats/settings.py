from copy import deepcopy
from threading import RLock
from typing import Callable

from flask_wtf import FlaskForm
from wtforms import BooleanField
from wtforms import IntegerField
from wtforms import StringField
from wtforms import SubmitField
from wtforms import FloatField


class Settings(FlaskForm):
    timeout = IntegerField(
        default=5,
        label="Timeout",
    )

    always_shift = BooleanField(
        default=True,
        label="Always shift",
        description="Speeds up search",
    )

    disable_shift = BooleanField(
        default=False,
        label="Disable shift",
        description="Disables shift",
    )

    allowed_moves = StringField(
        default="all",
        label="Allowed moves",
        description="Allowed moves in search, use to limit search space",
    )

    heuristic_weight = FloatField(
        default=1,
        label="Heuristic weight",
        description="Weight of heuristic in A* search",
    )

    simple_geometry = BooleanField(
        default=False,
        label="Simple geometry",
        description="Only use integer height/width in state",
    )

    object_hitbox = IntegerField(
        default=3,
        label="Object hitbox width",
        description="Width of object hitbox in pixels",
    )

    extend_deadly_hitbox = IntegerField(
        default=1,
        label="Extend deadly hitbox",
        description="Extend deadly hitbox by this amount of pixels",
    )

    submit_button = SubmitField("Submit Form")


__lock: RLock = RLock()
__settings: dict = dict()


def get_settings() -> dict:
    with __lock:
        return deepcopy(__settings)


def update_settings(upd: Callable[[dict], None]):
    with __lock:
        upd(__settings)
