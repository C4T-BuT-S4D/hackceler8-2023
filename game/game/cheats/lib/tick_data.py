from pydantic import BaseModel


class TickData(BaseModel):
    keys: list[int]
    random_seed: int

    force_keys: bool = False
