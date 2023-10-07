from copy import deepcopy
from threading import RLock
from typing import Callable

from wtforms import Form
from wtforms import BooleanField
from wtforms import FloatField
from wtforms import IntegerField
from wtforms import StringField
from wtforms import TextAreaField


class MacrosSettings(Form):
    title = "Macros"

    keys_to_press = TextAreaField(
        default="",
        label="Keys to press",
        description="Keys to press in JSON format",
    )

    cancel_macro_on_key_press = BooleanField(
        default=False,
        label="Cancel macros on keypress",
    )


class RenderingAndExtraSettings(Form):
    title = "Rendering & Extra"

    object_hitbox = IntegerField(
        default=3,
        label="Object hitbox width",
        description="Width of object hitbox in pixels",
    )

    draw_names = BooleanField(
        default=True,
        label="Draw object names",
    )

    draw_boxes = BooleanField(
        default=True,
        label="Draw object boxes",
    )

    draw_lines = BooleanField(
        default=True,
        label="Draw lines",
    )

    slow_ticks_count = IntegerField(
        default=1,
        label="Slow ticks count",
        description="Number of ticks to emulate in slow_ticks_mode",
    )

    random_seed = IntegerField(
        default=0,
        label="Random seed",
        description="Random seed for the game",
    )


class PathfindingSettings(Form):
    title = "Pathfinding"

    timeout = IntegerField(
        default=5,
        label="Timeout",
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

    state_batch_size = IntegerField(
        default=50000,
        label="State batch size",
        description="Number of states to process in one batch in parallel",
    )

    extend_deadly_hitbox = IntegerField(
        default=1,
        label="Extend deadly hitbox",
        description="Extend deadly hitbox by this amount of pixels",
    )

    always_shift = BooleanField(
        default=True,
        label="Always shift",
    )

    disable_shift = BooleanField(
        default=False,
        label="Disable shift",
    )

    simple_geometry = BooleanField(
        default=False,
        label="Simple geometry",
        description="Only use integer height/width in state",
    )

    validate_transitions = BooleanField(
        default=False,
        label="Validate transitions",
    )


settings_forms = [MacrosSettings, RenderingAndExtraSettings, PathfindingSettings]

__lock: RLock = RLock()
__settings: dict = dict()


def init_settings():
    forms = [form() for form in settings_forms]
    data = dict()
    for form in forms:
        data.update(**deepcopy(form.data))
    print(f"Initial settings: {data}")
    update_settings(lambda s: s.update(**data))


def get_settings() -> dict:
    with __lock:
        return deepcopy(__settings)


def update_settings(upd: Callable[[dict], None]):
    with __lock:
        upd(__settings)
