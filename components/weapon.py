import logging

from components.weapon_systems import cannon
from components.weapon_systems import gun


def parse_weapon(props: dict, coords):
    needed_props = ["type", "collectable"]
    for i in needed_props:
        if i not in props:
            logging.critical(f"Missing property {i}")
            return None
    logging.info(f"Adding new weapon type {props['type']}")
    match props["type"]:
        case "gun":
            return gun.Gun(coords, props["collectable"])
        case "ammo":
            return None
        case "cannon":
            return cannon.Cannon(coords, props["collectable"])
        case _:
            logging.error(f"Failed to parse object {props['type']}")
            return None
