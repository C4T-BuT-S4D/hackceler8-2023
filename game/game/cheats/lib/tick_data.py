from pydantic import BaseModel


class TickData(BaseModel):
    keys: list[int]
    random_seed: int
    text_input: str | None = None

    force_keys: bool = False
