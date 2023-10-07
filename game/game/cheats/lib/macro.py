from pydantic import BaseModel


class Macro(BaseModel):
    name: str
    keys: str
