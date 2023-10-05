from copy import deepcopy
from threading import RLock
from typing import Callable

from flask_wtf import FlaskForm
from wtforms import BooleanField
from wtforms import FloatField
from wtforms import IntegerField
from wtforms import StringField
from wtforms import SubmitField


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

    state_batch_size = IntegerField(
        default=50000,
        label="State batch size",
        description="Number of states to process in one batch in parallel",
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

    validate_transitions = BooleanField(
        default=False,
        label="Validate transitions",
        description="Validate transitions with rust",
    )

    draw_names = BooleanField(
        default=True, label="Draw names", description="Draw objects names"
    )

    draw_boxes = BooleanField(
        default=True, label="Draw boxes", description="Draw objects boxes"
    )

    draw_lines = BooleanField(
        default=True, label="Draw lines", description="Draw lines to important objects"
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
