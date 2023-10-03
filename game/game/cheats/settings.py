import dataclasses
from typing import Callable
from threading import RLock
from copy import deepcopy


@dataclasses.dataclass
class Settings:
    timeout: int = 5
    always_shift: bool = True


__lock: RLock = RLock()
__settings: Settings = Settings()


def get_settings() -> Settings:
    with __lock:
        return deepcopy(__settings)


def update_settings(upd: Callable[[Settings], None]):
    with __lock:
        upd(__settings)
