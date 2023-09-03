from enum import Enum

import arcade

from map_loading import tilemap


class GameMode(Enum):
    MODE_PLATFORMER = "platformer"
    MODE_SCROLLER = "scroller"


def load() -> dict:
    base_tilemap = tilemap.BasicTileMap("resources/maps/hackceler_map.tmx")
    base_scene = arcade.load_tilemap("resources/maps/hackceler_map.tmx")

    cctv_tilemap = tilemap.BasicTileMap("resources/levels/cctv/cctv_level.tmx")
    cctv_scene = arcade.load_tilemap("resources/levels/cctv/cctv_level.tmx")

    maps_dict = {
        "base": (base_tilemap, base_scene, GameMode.MODE_SCROLLER),
        "cctv": (cctv_tilemap, cctv_scene, GameMode.MODE_PLATFORMER),
        "rusty": (cctv_tilemap, cctv_scene, GameMode.MODE_PLATFORMER),
        "space": (cctv_tilemap, cctv_scene, GameMode.MODE_PLATFORMER),
        "water": (cctv_tilemap, cctv_scene, GameMode.MODE_PLATFORMER),
        "debug": (cctv_tilemap, cctv_scene, GameMode.MODE_PLATFORMER),
    }

    return maps_dict
