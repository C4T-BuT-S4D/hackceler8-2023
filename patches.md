####  

`game/game/ludicer_gui.py`

```python
    def on_mouse_press(self, x: int, y: int, button: int, modifiers: int):
    if self.game is None:
        return

    if button == arcade.MOUSE_BUTTON_LEFT:
        player = self.game.player

        target_x = x + self.camera.position.x
        target_y = y + self.camera.position.y

        logging.info(
            "mouse pressed at (%s, %s), player pos: (%s, %s), target: (%s, %s)",
            x,
            y,
            player.x,
            player.y,
            target_x,
            target_y,
        )

        initial_state = cheats_rust.PhysState(
            player=cheats_rust.PlayerState(
                x=player.x,
                y=player.y,
                vx=player.x_speed,
                vy=player.y_speed,
                in_the_air=player.in_the_air,
            )
        )
        static_state = cheats_rust.StaticState(
            objects=[
                cheats_rust.Hitbox(
                    outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline]
                )
                for o in self.game.tiled_map.static_objs
            ]
        )
        target_state = cheats_rust.PhysState(
            player=cheats_rust.PlayerState(
                x=target_x,
                y=target_y,
                vx=0,
                vy=0,
                in_the_air=False,
            )
        )

        initial_state = cheats_rust.PhysState(
            player=cheats_rust.PlayerState(
                x=544.817000,
                y=2592.000000,
                vx=0,
                vy=0,
                in_the_air=False,
            )
        )
        target_state = cheats_rust.PhysState(
            player=cheats_rust.PlayerState(
                x=2897.000000,
                y=4111.000000,
                vx=0,
                vy=0,
                in_the_air=False,
            )
        )

        timeout = 5
        if modifiers & arcade.key.MOD_ALT or modifiers & arcade.key.MOD_OPTION:
            timeout = 30

        path = cheats_rust.astar_search(
            initial_state=initial_state,
            target_state=target_state,
            static_state=static_state,
            timeout=timeout,
        )
        if not path:
            print("Path not found")
        else:
            self.force_next_keys = []
            for move in path:
                match move:
                    case cheats_rust.Move.W:
                        self.force_next_keys.append({"W"})
                    case cheats_rust.Move.A:
                        self.force_next_keys.append({"A"})
                    case cheats_rust.Move.D:
                        self.force_next_keys.append({"D"})
                    case cheats_rust.Move.WA:
                        self.force_next_keys.append({"W", "A"})
                    case cheats_rust.Move.WD:
                        self.force_next_keys.append({"W", "D"})
                    case cheats_rust.Move.NONE:
                        self.force_next_keys.append(set())
                    case _:
                        print("unknown move", move)
            print("path found", path, self.force_next_keys)

```

`game/game/ludicer_gui.py:on_update`

```python
if self.force_next_keys:
    for keys in self.force_next_keys[: self.force_next_keys_per_frame]:
        self.game.pressed_keys = {arcade.key.LSHIFT}
        for c in keys:
            match c:
                case "A":
                    self.game.pressed_keys.add(arcade.key.A)
                case "D":
                    self.game.pressed_keys.add(arcade.key.D)
                case "W":
                    self.game.pressed_keys.add(arcade.key.W)
                case _:
                    logging.fatal("unknown key %s", c)
        self.game.tick()

    self.force_next_keys = self.force_next_keys[
                           self.force_next_keys_per_frame:
                           ]
    self.game.pressed_keys = set()
else:
    self.game.tick()
```

`game/game/ludicer_gui.py:on_draw`

```python
for o in (
    self.game.tiled_map.objs
    + self.game.tiled_map.static_objs
    + self.game.tiled_map.moving_platforms
):
    color = None
    match o.nametype:
        case "Wall":
            color = arcade.color.RED
        case "MovingPlatform":
            color = arcade.color.RED_BROWN
        case "Door":
            color = arcade.color.BROWN
        case "NPC":
            color = arcade.color.YELLOW
        case "ExitArea":
            color = arcade.color.GREEN
        case "Player":
            color = arcade.color.BURNT_ORANGE
        case "Item":
            color = arcade.color.WHITE
        case "Arena":
            color = arcade.color.BLACK
        case _:
            print(f"skipped object {o.nametype}")

    if color:
        rect = o.get_rect()
        arcade.draw_lrtb_rectangle_outline(
            rect.x1,
            rect.x2,
            rect.y2,
            rect.y1,
            color,
            border_width=5,
        )
```

`game/game/ludicer_gui.py:center_camera_to_player`

```python
    def center_camera_to_player(self):
    screen_center_x = self.game.player.x - (self.camera.viewport_width / 2)
    screen_center_y = self.game.player.y - (self.camera.viewport_height / 2)

    # Don't let camera travel past 0
    # screen_center_x = max(screen_center_x, 0)
    # screen_center_y = max(screen_center_y, 0)

    # # additional controls to avoid showing around the map
    # max_screen_center_x = self.game.tiled_map.map_size_pixels[0] - self.SCREEN_WIDTH
    # max_screen_center_y = (
    #     self.game.tiled_map.map_size_pixels[1] - self.SCREEN_HEIGHT
    # )
    # screen_center_x = min(screen_center_x, max_screen_center_x)
    # screen_center_y = min(screen_center_y, max_screen_center_y)

    player_centered = Vec2(screen_center_x, screen_center_y)
    self.camera.move_to(player_centered)
```

`game/game/ludicer_gui.py:__init__`

```python
        self.force_next_keys = []
self.force_next_keys_per_frame = 1
```