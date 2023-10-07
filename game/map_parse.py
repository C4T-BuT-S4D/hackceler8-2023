import sys
import os

gamedir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "game")
sys.path.insert(0,gamedir)
os.chdir(gamedir)

import ludicer
lud = ludicer.Ludicer(None, True)
arenas = lud.arena_mapping
maps = lud.original_maps_dict
itemnames = [a.name for a in lud.global_match_items.items]
ans = dict()
for k,v in maps.items():
    if k == 'base':
        continue
    names = []
    for layer in v.tiled_map.parsed_map.layers:
        if hasattr(layer, 'tiled_objects'):
            to = layer.tiled_objects
            for obj in to:
                names.append(obj.name)
    ans[k] = [n for n in names if 'npc' in n or 'key' in n or 'item' in n or 'boss' in n or n in itemnames]
ars = []
for k,v in maps.items():
    if k != 'base':
        continue
    for layer in v.tiled_map.parsed_map.layers:
        if hasattr(layer, 'tiled_objects'):
            to = layer.tiled_objects
            for obj in to:
                if 'arena' not in obj.name:
                    continue
                ars.append(obj)
for obj in ars:
    x = (obj.coordinates.x - 100) / (980 - 100) * 4
    y = (obj.coordinates.y - 296) / (1182 - 296) * 4 + 1
    mapname = arenas[obj.name]
    print()
    print(obj.name, 'row', round(y), 'column', round(x), 'map', mapname)
    print(ans[mapname])
