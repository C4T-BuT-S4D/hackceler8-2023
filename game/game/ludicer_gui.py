# Copyright 2023 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import logging
import os
import uuid

import arcade
import cheats_rust
from arcade import gui
from PIL import Image
from pyglet.image import load as pyglet_load

import constants
import ludicer
from cheats.maps import render_finish
from cheats.maps import render_requested
from cheats.settings import get_settings
from map_loading.maps import GameMode

SCREEN_TITLE = "Hackceler8-23"


class QuitButton(arcade.gui.UIFlatButton):
    def on_click(self, _event: arcade.gui.UIOnClickEvent):
        arcade.exit()


class Hackceler8(arcade.Window):
    def __init__(self, net):
        arcade.load_font("resources/textbox/Pixel.ttf")
        self.SCREEN_WIDTH = constants.SCREEN_WIDTH
        self.SCREEN_HEIGHT = constants.SCREEN_HEIGHT
        self.SCREEN_TITLE = SCREEN_TITLE
        super().__init__(
            self.SCREEN_WIDTH, self.SCREEN_HEIGHT, self.SCREEN_TITLE, resizable=True
        )
        self.set_icon(pyglet_load("resources/character/32bit/main32.PNG"))

        self.v_box = None

        self.net = net
        self.game = None

        # GUI management
        self.main_menu_manager = gui.UIManager()
        self.main_menu_manager.enable()

        self.force_next_keys = []
        self.force_next_keys_per_frame = 1

        self.camera = None
        self.gui_camera = None  # For stationary objects.

        # for rendering the map. create once to avoid framebuffer object leak.
        self.map_atlas = arcade.TextureAtlas((1, 1))

        # cache from width, height to texture used for re-rendering maps of the same size
        self.map_texture_cache: dict[(int, int), arcade.Texture] = dict()

        self._setup()
        self.show_menu()

        self.force_next_keys = []
        self.force_next_keys_per_frame = 1

        self.slow_ticks_mode = False

    def show_menu(self):
        self.main_menu_manager.enable()
        self.main_menu_manager.clear()
        arcade.set_background_color(arcade.color.DARK_BLUE_GRAY)

        self.v_box = arcade.gui.UIBoxLayout()
        start_button = arcade.gui.UIFlatButton(
            text="START GAME", width=200, style={"font_name": constants.FONT_NAME}
        )
        self.v_box.add(start_button.with_space_around(bottom=20))
        quit_button = QuitButton(
            text="QUIT", width=200, style={"font_name": constants.FONT_NAME}
        )
        self.v_box.add(quit_button)
        start_button.on_click = self.on_click_start

        w = arcade.gui.UIAnchorWidget(
            anchor_x="center_x", anchor_y="center_y", child=self.v_box
        )
        self.main_menu_manager.add(w)

    def on_click_start(self, _event):
        arcade.start_render()
        arcade.draw_text(
            text="LOADING GAME",
            start_x=0,
            start_y=self.SCREEN_HEIGHT // 2,
            color=arcade.csscolor.GRAY,
            font_name=constants.FONT_NAME,
            font_size=50,
            width=self.SCREEN_WIDTH,
            align="center",
        )
        arcade.draw_text(
            text="(THIS SHOULD TAKE LIKE 40 SECONDS)",
            start_x=0,
            start_y=self.SCREEN_HEIGHT // 2 - 100,
            color=arcade.csscolor.GRAY,
            font_name=constants.FONT_NAME,
            font_size=20,
            width=self.SCREEN_WIDTH,
            align="center",
        )
        arcade.finish_render()
        self.start_game()
        self.main_menu_manager.disable()

    def _setup(self):
        self.camera = arcade.Camera(self.width, self.height)
        self.gui_camera = arcade.Camera(self.width, self.height)

    def start_game(self):
        self.game = ludicer.Ludicer(self.net, is_server=False)

    # This method is automatically called when the window is resized.
    def on_resize(self, width, height):
        self.camera.resize(width, height)
        self.gui_camera.resize(width, height)

        # Call the parent. Failing to do this will mess up the coordinates,
        # and default to 0,0 at the center and the edges being -1 to 1.
        super().on_resize(width, height)

        logging.debug(f"resized to: {width}, {height}")
        if self.game is None:
            return
        self.center_camera_to_player()

    def center_camera_to_player(self):
        screen_center_x = self.game.player.x - (self.camera.viewport_width / 2)
        screen_center_y = self.game.player.y - (self.camera.viewport_height / 2)

        # Don't let camera travel past 0
        # screen_center_x = max(screen_center_x, 0)
        # screen_center_y = max(screen_center_y, 0)
        #
        # # additional controls to avoid showing around the map
        # max_screen_center_x = self.game.tiled_map.map_size_pixels[
        #                           0] - self.camera.viewport_width
        # max_screen_center_y = self.game.tiled_map.map_size_pixels[
        #                           1] - self.camera.viewport_height
        # screen_center_x = min(screen_center_x, max_screen_center_x)
        # screen_center_y = min(screen_center_y, max_screen_center_y)

        player_centered = screen_center_x, screen_center_y
        self.camera.move_to(player_centered)

    def on_draw(self):
        self.gui_camera.use()

        if self.game is None:
            self.main_menu_manager.draw()
            return

        self.camera.use()
        arcade.start_render()

        self.draw_level()

        self.gui_camera.use()

        if self.game.danmaku_system is not None:
            self.game.danmaku_system.draw_gui()

        arcade.draw_text(
            "HEALTH: %.02f" % self.game.player.health,
            10,
            10,
            arcade.csscolor.RED,
            18,
            font_name=constants.FONT_NAME,
        )

        if self.game.player.dead:
            arcade.draw_text(
                "YOU DIED",
                640,
                640,
                arcade.csscolor.RED,
                64,
                font_name=constants.FONT_NAME,
                anchor_x="center",
                anchor_y="center",
            )

        if self.game.won:
            arcade.draw_text(
                "CONGRATULATIONS, YOU WIN!",
                450,
                600,
                arcade.csscolor.WHITE,
                18,
                font_name=constants.FONT_NAME,
            )
            arcade.draw_text(
                "TOTAL PLAY TIME: %s" % self.game.play_time_str(),
                455,
                550,
                arcade.csscolor.WHITE,
                18,
                font_name=constants.FONT_NAME,
            )

        if self.game.cheating_detected:
            arcade.draw_text(
                "OUT OF SYNC, CHEATING DETECTED",
                400,
                600,
                arcade.csscolor.ORANGE,
                18,
                font_name=constants.FONT_NAME,
            )

        if self.game.display_inventory:
            if (
                not self.game.prev_display_inventory
                or self.game.inventory.display_time != self.game.play_time_str()
            ):
                self.game.inventory.update_display()
            self.game.inventory.draw()

        if self.game.textbox is not None:
            self.game.textbox.draw()
        if self.game.map_switch is not None:
            self.game.map_switch.draw()

        arcade.finish_render()

    def draw_level(self):
        self.game.tiled_map.texts[1].draw_scaled(0, 0)
        if self.game.prerender is None:
            self.game.scene.draw()
        else:
            arcade.draw_lrwh_rectangle_textured(
                0,
                0,
                self.game.prerender.width,
                self.game.prerender.height,
                self.game.prerender,
            )

        for o in self.game.dynamic_artifacts:
            o.draw()

        for o in self.game.tiled_map.moving_platforms:
            o.draw()

        for o in self.game.static_objs:
            o.draw()

        for o in self.game.objects:
            if o.render_above_player:
                continue
            o.draw()

        if self.game.grenade_system:
            self.game.grenade_system.draw()
        self.game.combat_system.draw()

        if self.game.player is not None:
            self.game.player.draw()

        for o in self.game.objects:
            if not o.render_above_player:
                continue
            o.draw()

        if self.game.danmaku_system is not None:
            self.game.danmaku_system.draw()

        for o in (
            self.game.tiled_map.objs
            + self.game.tiled_map.static_objs
            + self.game.tiled_map.moving_platforms
            + self.game.tiled_map.dynamic_artifacts
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
                case "BossGate":
                    color = arcade.color.PURPLE
                case "Portal":
                    color = arcade.color.BLUE
                case "Toggle":
                    color = arcade.color.ORANGE
                case "Buffer":
                    color = arcade.color.PINK
                case "Enemy":
                    color = arcade.color.RED_DEVIL
                case "Spike":
                    color = arcade.color.RED_DEVIL
                case "SpeedTile":
                    color = arcade.color.BLUE_GREEN
                case "Switch":
                    color = arcade.color.CADMIUM_GREEN
                case "Boss":
                    color = arcade.color.CYAN
                case _:
                    print(f"skipped object {o.nametype}")

            if color:
                rect = o.get_rect()
                arcade.draw_lrtb_rectangle_outline(
                    rect.x1(),
                    rect.x2(),
                    rect.y2(),
                    rect.y1(),
                    color,
                    border_width=1,
                )

    def render_map(self):
        tiled_map_size = self.game.tiled_map.parsed_map.map_size
        tile_size = self.game.tiled_map.parsed_map.tile_size

        map_width = round(tiled_map_size.width * tile_size.width)
        map_height = round(tiled_map_size.height * tile_size.height)

        map_texture = self.map_texture_cache.get((map_width, map_height))
        if map_texture is None:
            map_texture = arcade.Texture.create_empty(
                str(uuid.uuid4()), (map_width, map_height)
            )
            self.map_texture_cache[(map_width, map_height)] = map_texture

        self.map_atlas.add(map_texture)

        with self.map_atlas.render_into(map_texture) as fb:
            fb.clear()
            self.draw_level()
            data = fb.read(viewport=fb.viewport, components=3)
            image = Image.frombytes("RGB", (map_width, map_height), bytes(data))
            image.save(
                os.path.join(
                    os.path.dirname(__file__), "cheats", "static", "current-map.png"
                )
            )

    def on_update(self, _delta_time: float):
        if self.game is None:
            return

        if render_requested():
            self.render_map()
            render_finish()

        if self.slow_ticks_mode:
            return

        if self.force_next_keys:
            for keys in self.force_next_keys[: self.force_next_keys_per_frame]:
                self.game.raw_pressed_keys = keys
                self.game.tick()

            self.force_next_keys = self.force_next_keys[
                self.force_next_keys_per_frame :
            ]
            self.game.raw_pressed_keys = set()
        else:
            self.game.tick()

        self.center_camera_to_player()

    def on_key_press(self, symbol: int, modifiers: int):
        if self.game is None:
            return

        if symbol == arcade.key.J and modifiers & arcade.key.MOD_CTRL:
            self.slow_ticks_mode = True
            logging.info("Slow ticks mode: %s", self.slow_ticks_mode)
            return

        if symbol == arcade.key.K and modifiers & arcade.key.MOD_CTRL:
            self.slow_ticks_mode = False
            logging.info("Slow ticks mode: %s", self.slow_ticks_mode)
            return

        if (
            self.slow_ticks_mode
            and symbol == arcade.key.BACKSPACE
            and self.game is not None
        ):
            self.game.tick()
            self.center_camera_to_player()
            print(
                f"player state: {self.game.player.x=} {self.game.player.y=} "
                f"{self.game.player.x_speed=} {self.game.player.y_speed=} {self.game.player.push_speed=} "
                f"{self.game.player.in_the_air=} {self.game.player.dead=} "
                f"{self.game.player.health=} "
            )
            return

        if symbol == arcade.key.M:
            logging.info("Showing menu")
            self.show_menu()
            return

        if self.force_next_keys:
            self.force_next_keys = []

        self.game.raw_pressed_keys.add(symbol)

    def on_key_release(self, symbol: int, _modifiers: int):
        if self.game is None:
            return
        if symbol in self.game.raw_pressed_keys:
            self.game.raw_pressed_keys.remove(symbol)

    def on_mouse_press(self, x: int, y: int, button: int, modifiers: int):
        if self.game is None:
            return

        if button == arcade.MOUSE_BUTTON_LEFT and modifiers & arcade.key.MOD_CTRL:
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

            mode = None
            match self.game.current_mode:
                case GameMode.MODE_SCROLLER:
                    mode = cheats_rust.GameMode.Scroller
                case GameMode.MODE_PLATFORMER:
                    mode = cheats_rust.GameMode.Platformer

            cheat_settings = get_settings()
            print("cheat settings:", cheat_settings)

            allowed_moves = []
            if cheat_settings["allowed_moves"].lower() not in {"", "all"}:
                moves = cheat_settings["allowed_moves"].upper().split(",")
                for move in moves:
                    allowed_moves.append(getattr(cheats_rust.Move, move))

            settings = cheats_rust.SearchSettings(
                mode=mode,
                timeout=cheat_settings["timeout"],
                always_shift=cheat_settings["always_shift"],
                disable_shift=cheat_settings["disable_shift"],
                allowed_moves=allowed_moves,
                enable_vpush=any(o.nametype == "SpeedTile" for o in self.game.objects),
            )

            print(
                "Static objects:", {o.nametype for o in self.game.tiled_map.static_objs}
            )
            print("Game objects:", {o.nametype for o in self.game.objects})

            static_objects = [
                (
                    cheats_rust.Hitbox(
                        outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline],
                    ),
                    cheats_rust.ObjectType.Wall,
                )
                for o in self.game.tiled_map.static_objs
            ]
            static_objects += [
                (
                    cheats_rust.Hitbox(
                        outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline],
                    ),
                    cheats_rust.ObjectType.Spike,
                )
                for o in self.game.objects
                if o.nametype == "Spike"
            ]
            static_objects += [
                (
                    cheats_rust.Hitbox(
                        outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline],
                    ),
                    cheats_rust.ObjectType.SpeedTile,
                )
                for o in self.game.objects
                if o.nametype == "SpeedTile"
            ]
            enviroments = [
                cheats_rust.EnvModifier(
                    hitbox=cheats_rust.Hitbox(
                        outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline],
                    ),
                    jump_speed=o.modifier.jump_speed,
                    jump_height=o.modifier.jump_height,
                    walk_speed=o.modifier.walk_speed,
                    run_speed=o.modifier.run_speed,
                    gravity=o.modifier.gravity,
                    jump_override=o.modifier.jump_override,
                )
                for o in self.game.physics_engine.env_tiles
            ]

            player_direction = None
            match player.direction:
                case player.DIR_N:
                    player_direction = cheats_rust.Direction.N
                case player.DIR_E:
                    player_direction = cheats_rust.Direction.E
                case player.DIR_S:
                    player_direction = cheats_rust.Direction.S
                case player.DIR_W:
                    player_direction = cheats_rust.Direction.W

            initial_state = cheats_rust.PhysState(
                player=cheats_rust.PlayerState(
                    x=player.x,
                    y=player.y,
                    vx=player.x_speed,
                    vy=player.y_speed,
                    vpush=player.push_speed,
                    direction=player_direction,
                    can_control_movement=player.can_control_movement,
                    in_the_air=player.in_the_air,
                ),
                settings=settings.physics_settings(),
            )
            static_state = cheats_rust.StaticState(
                objects=static_objects,
                environments=enviroments,
            )
            target_state = cheats_rust.PhysState(
                player=cheats_rust.PlayerState(
                    x=target_x,
                    y=target_y,
                    vx=0,
                    vy=0,
                    vpush=0,
                    direction=player_direction,
                    can_control_movement=False,
                    in_the_air=False,
                ),
                settings=settings.physics_settings(),
            )

            path = cheats_rust.astar_search(
                settings=settings,
                initial_state=initial_state,
                target_state=target_state,
                static_state=static_state,
            )
            if not path:
                print("Path not found")
            else:
                self.force_next_keys = []
                for move, shift in path:
                    match move:
                        case cheats_rust.Move.W:
                            moves = {arcade.key.W}
                        case cheats_rust.Move.A:
                            moves = {arcade.key.A}
                        case cheats_rust.Move.D:
                            moves = {arcade.key.D}
                        case cheats_rust.Move.WA:
                            moves = {arcade.key.W, arcade.key.A}
                        case cheats_rust.Move.WD:
                            moves = {arcade.key.W, arcade.key.D}
                        case cheats_rust.Move.S:
                            moves = {arcade.key.S}
                        case cheats_rust.Move.SA:
                            moves = {arcade.key.S, arcade.key.A}
                        case cheats_rust.Move.SD:
                            moves = {arcade.key.S, arcade.key.D}
                        case cheats_rust.Move.NONE:
                            moves = set()
                        case _:
                            print("unknown move", move)
                            continue

                    if shift:
                        moves.add(arcade.key.LSHIFT)

                    self.force_next_keys.append(moves)

                print("path found", path)
