from typing import Callable
from threading import RLock
from copy import deepcopy

from flask_wtf import FlaskForm
from wtforms import StringField, IntegerField, BooleanField, SubmitField


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

    submit_button = SubmitField("Submit Form")


__lock: RLock = RLock()
__settings: dict = dict()


def get_settings() -> dict:
    with __lock:
        return deepcopy(__settings)


def update_settings(upd: Callable[[dict], None]):
    with __lock:
        upd(__settings)
