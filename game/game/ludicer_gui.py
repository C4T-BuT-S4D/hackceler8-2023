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
import ast
import logging
import os
import shutil
import time
import traceback
import uuid

import arcade
import cheats_rust
import pyglet
import pyglet.media
from arcade import gui
from PIL import Image
from pyglet.image import load as pyglet_load

import cheats.lib.geom as cheats_geom
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

        self.force_movement_keys = []

        self.add_movement_keys = []

        self.num_weapon_shifts = -1
        self.auto_weapon_shooting = False
        self.auto_danmaku_shooting = False

        self.draws = []

        self.camera = None
        self.gui_camera = None  # For stationary objects.

        # for rendering the map. create once to avoid framebuffer object leak.
        self.map_atlas = arcade.TextureAtlas((1, 1))

        # cache from width, height to texture used for re-rendering maps of the same size
        self.map_texture_cache: dict[(int, int), arcade.Texture] = dict()

        self._setup()
        self.show_menu()

        self.slow_ticks_mode = False
        logging.info("Using audio driver {}".format(pyglet.media.get_audio_driver()))

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

    def record_draw(self):
        now = time.time()
        self.draws.append(now)
        while self.draws and now - self.draws[0] > 3:
            self.draws.pop(0)

    def on_draw(self):
        self.record_draw()

        self.gui_camera.use()

        if self.game is None:
            arcade.start_render()
            self.main_menu_manager.draw()
            return

        self.camera.use()
        arcade.start_render()

        if self.game.cheating_detected:
            self.gui_camera.use()
            arcade.draw_text(
                "OUT OF SYNC, CHEATING DETECTED",
                400,
                600,
                arcade.csscolor.ORANGE,
                18,
                font_name=constants.FONT_NAME,
            )
            arcade.finish_render()
            return

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

        if len(self.draws) > 1:
            arcade.draw_text(
                "FPS: %.02f" % (len(self.draws) / (self.draws[-1] - self.draws[0])),
                300,
                10,
                arcade.csscolor.WHITE,
                18,
                font_name=constants.FONT_NAME,
            )

        if self.game.danmaku_system and self.game.danmaku_system.boss_health:
            arcade.draw_text(
                "BOSS HEALTH: %.02f" % self.game.danmaku_system.boss_health,
                500,
                10,
                arcade.csscolor.BLUE,
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
                "CONGRATULATIONS, YOU BEAT ALL %d CHALLENGES!"
                % (len(self.game.global_match_items.items)),
                300,
                600,
                arcade.csscolor.WHITE,
                18,
                font_name=constants.FONT_NAME,
            )

        # if self.game.cheating_detected:
        #     arcade.draw_text(
        #         "OUT OF SYNC, CHEATING DETECTED",
        #         400,
        #         600,
        #         arcade.csscolor.ORANGE,
        #         18,
        #         font_name=constants.FONT_NAME,
        #     )

        if self.game.display_inventory:
            if (
                not self.game.prev_display_inventory
                or self.game.inventory.display_time != self.game.play_time_str()
            ):
                self.game.inventory.update_display()
            self.game.inventory.draw()

        if self.game.textbox is not None:
            self.game.textbox.draw()
        if self.game.jam_session is not None:
            self.game.jam_session.draw()
            self.camera.use()
            self.game.jam_session.draw_notes()
            self.gui_camera.use()
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

        if self.game.logic_engine is not None:
            self.game.logic_engine.draw()

        if self.game.grenade_system:
            self.game.grenade_system.draw()
        self.game.combat_system.draw()

        if self.game.player is not None:
            self.game.player.draw()
            # this draws the outline

        for o in self.game.objects:
            if not o.render_above_player:
                continue
            o.draw()

        if self.game.danmaku_system is not None:
            self.game.danmaku_system.draw()

        cheats_settings = get_settings()
        for o in (
            self.game.tiled_map.objs
            + self.game.static_objs
            + self.game.tiled_map.moving_platforms
            + self.game.tiled_map.dynamic_artifacts
            + self.game.combat_system.original_weapons
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
                    if o.on:
                        color = arcade.color.RED_DEVIL
                    else:
                        color = arcade.color.PALE_RED_VIOLET
                case "SpeedTile":
                    color = arcade.color.BLUE_GREEN
                case "Switch":
                    color = arcade.color.CADMIUM_GREEN
                case "Boss":
                    color = arcade.color.CYAN
                case "Weapon":
                    color = arcade.color.CORAL
                case "Ouch":
                    color = arcade.color.CADMIUM_GREEN
                case "Fire":
                    color = arcade.color.ATOMIC_TANGERINE
                case "Duck":
                    color = arcade.color.YELLOW_ROSE
                case "Brainduck":
                    color = arcade.color.AMARANTH
                case _:
                    logging.debug(f"skipped object {o.nametype}")

            if color:
                rect = o.get_rect()

                if cheats_settings["draw_boxes"]:
                    arcade.draw_lrtb_rectangle_outline(
                        rect.x1(),
                        rect.x2(),
                        rect.y2(),
                        rect.y1(),
                        color,
                        border_width=cheats_settings["object_hitbox"],
                    )

                if cheats_settings["draw_names"] and o.nametype not in {"Wall"}:
                    text = f"{o.nametype} | {o.name}"
                    if hasattr(o, "health"):
                        text += f" | {o.health}"
                    arcade.draw_text(text, rect.x1(), rect.y2() + 6, color)

                if cheats_settings["draw_lines"]:
                    if o.nametype == "Item":
                        line_color = getattr(
                            arcade.color, (o.color or "").upper(), None
                        )
                        if line_color is None:
                            logging.debug(
                                f"failed to get line color for item of color {o.color}, will use the default color"
                            )
                            line_color = color

                        arcade.draw_line(
                            start_x=self.game.player.x,
                            start_y=self.game.player.y,
                            end_x=abs(rect.x1() + rect.x2()) / 2,
                            end_y=abs(rect.y1() + rect.y2()) / 2,
                            color=line_color,
                            line_width=2,
                        )

                    if o.nametype == "Portal":
                        arcade.draw_line(
                            start_x=o.x,
                            start_y=o.y,
                            end_x=o.dest.x,
                            end_y=o.dest.y,
                            color=arcade.color.PURPLE,
                            line_width=2,
                        )
                    # render lines to something else as well

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

    def tick_game_with_shooting(self):
        added_keys = set()
        weapon_firing_enabled = self.num_weapon_shifts >= 0

        # stop firing if the player or his weapons are gone (map switched, died, ended level)
        if (
            weapon_firing_enabled
            and not self.game
            or not self.game.player
            or len(self.game.player.weapons) < 1
        ):
            self.num_weapon_shifts = -1

        # on correct weapon, shoot it if we can
        if (
            self.num_weapon_shifts == 0
            and self.game.player.weapons[0].cool_down_timer == 0
            and (arcade.key.SPACE not in self.game.prev_pressed_keys)
        ):
            added_keys.add(arcade.key.SPACE)
            self.num_weapon_shifts = -1

        # planning to shoot some weapon but didn't reach it yet
        if self.num_weapon_shifts > 0:
            skip_shift = (
                arcade.key.Q in self.game.prev_pressed_keys
                or arcade.key.SPACE in self.game.prev_pressed_keys
            )

            if not skip_shift:
                # drop current weapon, will select next one
                added_keys.add(arcade.key.Q)
                # next weapon selected, one less to shift through
                self.num_weapon_shifts -= 1
                # pick up dropped weapon, will be placed at the end
                added_keys.add(arcade.key.SPACE)

            # just shifted to the correct weapon and it's cooldown is 0, it will fire immediately
            if (
                self.num_weapon_shifts == 0
                and self.game.player.weapons[0].cool_down_timer == 0
            ):
                self.num_weapon_shifts = -1

        if self.auto_danmaku_shooting:
            added_keys.add(arcade.key.SPACE)

        # add temporary pressed keys for this single tick
        for key in added_keys:
            self.game.raw_pressed_keys.add(key)

        self.game.tick()

        for key in added_keys:
            self.game.raw_pressed_keys.remove(key)

        # check auto weapon shooting after ticking because something might've broken already,
        # e.g. user might've thrown all the weapons out
        if self.auto_weapon_shooting and not (
            self.game is not None
            and self.game.player is not None
            and len(self.game.player.weapons) > 0
        ):
            self.auto_weapon_shooting = False

        # try to find the next weapon to shoot if shooting is enabled
        if weapon_firing_enabled and self.auto_weapon_shooting:
            min_index = self.closest_shootable_weapon()
            if min_index >= 0:
                self.num_weapon_shifts = min_index

    def tick_game_with_movement_and_shooting(self):
        keys_to_restore = None
        if self.force_movement_keys:
            keys_to_restore = self.game.raw_pressed_keys.copy()
            self.game.raw_pressed_keys = set(self.force_movement_keys[0])
            self.force_movement_keys = self.force_movement_keys[1:]
        elif self.add_movement_keys:
            keys_to_restore = self.game.raw_pressed_keys.copy()
            self.game.raw_pressed_keys |= set(self.add_movement_keys[0])
            self.add_movement_keys = self.add_movement_keys[1:]

        settings, state, static_state = self.to_rust_state()
        keys = set(self.game.raw_pressed_keys)  # copy
        if arcade.key.LSHIFT in keys:
            shift = True
            keys.remove(arcade.key.LSHIFT)
        else:
            shift = False
        keys &= {arcade.key.W, arcade.key.A, arcade.key.S, arcade.key.D}
        if keys == {arcade.key.W}:
            move = cheats_rust.Move.W
        elif keys == {arcade.key.A}:
            move = cheats_rust.Move.A
        elif keys == {arcade.key.D}:
            move = cheats_rust.Move.D
        elif keys == {arcade.key.W, arcade.key.A}:
            move = cheats_rust.Move.WA
        elif keys == {arcade.key.W, arcade.key.D}:
            move = cheats_rust.Move.WD
        elif keys == {arcade.key.S}:
            move = cheats_rust.Move.S
        elif keys == {arcade.key.S, arcade.key.A}:
            move = cheats_rust.Move.SA
        elif keys == {arcade.key.S, arcade.key.D}:
            move = cheats_rust.Move.SD
        elif keys == set():
            move = cheats_rust.Move.NONE
        else:
            self.tick_game_with_shooting()
            if keys_to_restore is not None:
                self.game.raw_pressed_keys = keys_to_restore
            return

        if get_settings()["validate_transitions"]:
            expected = cheats_rust.get_transition(
                settings, static_state, state, move, shift
            )

        self.tick_game_with_shooting()

        if keys_to_restore is not None:
            self.game.raw_pressed_keys = keys_to_restore

        if get_settings()["validate_transitions"]:
            attrs = [("x", "x"), ("y", "y"), ("x_speed", "vx"), ("y_speed", "vy")]
            vals = [
                (getattr(self.game.player, k1), getattr(expected, k2))
                for k1, k2 in attrs
            ]
            if not all(x == y for x, y in vals):
                print("incorrect transition:")
                for (k1, k2), (v1, v2) in zip(attrs, vals):
                    print(k1, "predicted", v2, "got", v1)
                print()

    def on_update(self, _delta_time: float):
        if self.game is None:
            return

        if render_requested():
            self.render_map()
            render_finish()

        if self.slow_ticks_mode:
            return

        self.tick_game_with_movement_and_shooting()
        self.center_camera_to_player()

    def closest_shootable_weapon(self) -> int:
        skip_first_switch = (
            arcade.key.Q in self.game.prev_pressed_keys
            or arcade.key.SPACE in self.game.prev_pressed_keys
        )

        weapons = self.game.player.weapons

        if not weapons[0].equipped and not self.select_first_weapon():
            return -1

        min_tics = self.game.player.weapons[0].cool_down_timer
        min_index = 0

        for index in range(1, len(weapons)):
            weapon = weapons[index]

            switch_time = skip_first_switch + index * 2 - 1
            wait_time = max(0, weapon.cool_down_timer - switch_time)
            tics = switch_time + wait_time

            if tics < min_tics:
                min_tics = tics
                min_index = index

        return min_index

    def select_first_weapon(self) -> bool:
        original_pressed_keys = self.game.raw_pressed_keys.copy()

        weapons = self.game.player.weapons
        equipped_index = -1
        for i, weapon in enumerate(weapons):
            if weapon.equipped:
                equipped_index = i
                break

        if equipped_index == -1:
            logging.error(
                "couldn't find equipped weapon despite user having >0 weapons"
            )
            return False

        self.game.raw_pressed_keys = {arcade.key.P}
        self.game.tick()
        time.sleep(pyglet.clock.get_sleep_time(True))

        for i in range(equipped_index):
            self.game.raw_pressed_keys = {arcade.key.W}
            self.game.tick()
            time.sleep(pyglet.clock.get_sleep_time(True))

        self.game.raw_pressed_keys = {arcade.key.P}
        self.game.tick()
        time.sleep(pyglet.clock.get_sleep_time(True))

        self.game.raw_pressed_keys = original_pressed_keys

    def on_key_press(self, symbol: int, modifiers: int):
        if self.game is None:
            return

        if symbol == arcade.key.EQUAL:
            self.slow_ticks_mode = not self.slow_ticks_mode
            logging.info("Slow ticks mode: %s", self.slow_ticks_mode)
            return

        if (
            (modifiers & arcade.key.MOD_ALT) != 0
            or (modifiers & arcade.key.MOD_OPTION) != 0
        ) and (
            symbol
            in {
                arcade.key.KEY_1,
                arcade.key.KEY_2,
                arcade.key.KEY_3,
                arcade.key.KEY_4,
                arcade.key.KEY_5,
                arcade.key.KEY_6,
                arcade.key.KEY_7,
                arcade.key.KEY_8,
                arcade.key.KEY_9,
            }
        ):
            settings = get_settings()
            macros = settings["macros"]
            if not macros:
                return

            try:
                macros = ast.literal_eval(macros)
                assert type(macros) == list

                keys_to_press = macros[symbol - arcade.key.KEY_1]

                self.add_movement_keys = []
                for keys in keys_to_press:
                    # keys: str (single symbol)
                    # keys: list[str] (multiple symbols)
                    # keys: list[int]
                    if isinstance(keys, str):
                        keys = keys.split(",")
                    # keys: list[str] (multiple symbols)
                    # keys: list[int]

                    cur_keys = set()
                    for key in keys:
                        if isinstance(key, str):
                            key = getattr(arcade.key, key)
                        cur_keys.add(key)
                    self.add_movement_keys.append(cur_keys)
            except Exception as e:
                print(f"bad macros: {macros}: {e}\n{traceback.format_exc()}")
            finally:
                return

        # enable automatic shooting which will always select the next
        # weapon to shoot once we've shot the current one
        auto_weapon_shooting_keys = (
            symbol == arcade.key.BRACKETRIGHT or symbol == arcade.key.BRACERIGHT
        )
        if auto_weapon_shooting_keys:
            self.auto_weapon_shooting = not self.auto_weapon_shooting

        # disable automatic shooting when using semi-automatic one
        semiauto_weapon_shooting = (
            symbol == arcade.key.BRACKETLEFT or symbol == arcade.key.BRACELEFT
        )
        if semiauto_weapon_shooting:
            self.auto_weapon_shooting = False

        if (
            (semiauto_weapon_shooting or self.auto_weapon_shooting)
            and self.game is not None
            and self.game.player is not None
            and len(self.game.player.weapons) > 0
        ):
            # find the closest shootable gun and label it in order to try switching to it
            min_index = self.closest_shootable_weapon()
            if min_index >= 0:
                self.num_weapon_shifts = min_index

        # if started dropping weapons or shooting manually, then cancel the switching
        if symbol == arcade.key.Q or symbol == arcade.key.SPACE:
            self.num_weapon_shifts = -1

        # toggle shooting for danmaku
        if self.game.danmaku_system and auto_weapon_shooting_keys:
            self.auto_danmaku_shooting = not self.auto_danmaku_shooting

        if (
            self.slow_ticks_mode
            and symbol == arcade.key.BACKSPACE
            and self.game is not None
        ):
            for _ in range(get_settings()["slow_ticks_count"]):
                self.tick_game_with_movement_and_shooting()

            self.center_camera_to_player()
            print(
                f"player state: {self.game.player.x=} {self.game.player.y=} "
                f"{self.game.player.x_speed=} {self.game.player.y_speed=} {self.game.player.push_speed=} "
                f"{self.game.player.in_the_air=} {self.game.player.dead=} "
                f"{self.game.player.health=} "
            )
            return

        if symbol == arcade.key.AMPERSAND:
            original_pressed_keys = self.game.raw_pressed_keys.copy()

            for map in list(self.game.arena_mapping.values()) + ["base"]:
                if map in ["boss", "danmaku"]:
                    logging.info("Skipping render of boss map {map}")
                    continue

                logging.info(f"Switching to Map {map}")
                self.game.switch_maps(map)

                start = time.time()
                while time.time() - start < 5:
                    self.game.raw_pressed_keys = set()
                    self.game.tick()
                    self.on_draw()

                self.render_map()
                cheats_path = os.path.join(
                    os.path.dirname(__file__), "cheats", "static"
                )
                shutil.move(
                    os.path.join(cheats_path, "current-map.png"),
                    os.path.join(cheats_path, f"map-{map}.png"),
                )

            logging.info("Rendered all maps!")

            self.game.raw_pressed_keys = original_pressed_keys

        if symbol == arcade.key.M:
            return

        if self.force_movement_keys:
            self.force_movement_keys = []

        if self.add_movement_keys and get_settings()["cancel_macro_on_key_press"]:
            self.add_movement_keys = []

        self.game.raw_pressed_keys.add(symbol)

    def on_key_release(self, symbol: int, _modifiers: int):
        if self.game is None:
            return
        if symbol in self.game.raw_pressed_keys:
            self.game.raw_pressed_keys.remove(symbol)

    def to_rust_state(self):
        mode = None
        player = self.game.player
        match self.game.current_mode:
            case GameMode.MODE_SCROLLER:
                mode = cheats_rust.GameMode.Scroller
            case GameMode.MODE_PLATFORMER:
                mode = cheats_rust.GameMode.Platformer

        cheat_settings = get_settings()
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
            heuristic_weight=cheat_settings["heuristic_weight"],
            enable_vpush=any(o.nametype == "SpeedTile" for o in self.game.objects),
            simple_geometry=cheat_settings["simple_geometry"],
            state_batch_size=cheat_settings["state_batch_size"],
            speed_multiplier=player.speed_multiplier,
            jump_multiplier=player.jump_multiplier,
        )

        static_objects = [
            (
                cheats_rust.Hitbox(
                    outline=[cheats_rust.Pointf(x=p.x, y=p.y) for p in o.outline],
                ),
                cheats_rust.ObjectType.Wall,
            )
            for o in self.game.tiled_map.static_objs
        ]

        deadly_objects_type = {
            "Spike": cheats_rust.ObjectType.Spike,
            "Arena": cheats_rust.ObjectType.Arena,
            "Portal": cheats_rust.ObjectType.Portal,
        }
        static_objects += [
            (
                cheats_geom.rect_to_rust_hitbox(
                    o.get_rect().expand(cheat_settings["extend_deadly_hitbox"])
                ),
                deadly_objects_type[o.nametype],
            )
            for o in self.game.objects + self.game.static_objs
            if o.nametype in deadly_objects_type
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
                hx_min=player.get_leftmost_point(),
                hx_max=player.get_rightmost_point(),
                hy_min=player.get_lowest_point(),
                hy_max=player.get_highest_point(),
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
        return settings, initial_state, static_state

    def on_mouse_press(self, x: int, y: int, button: int, modifiers: int):
        if self.game is None:
            return
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
        if button == arcade.MOUSE_BUTTON_LEFT and modifiers & arcade.key.MOD_CTRL:
            print(
                "Static objects:", {o.nametype for o in self.game.tiled_map.static_objs}
            )
            print("Game objects:", {o.nametype for o in self.game.objects})
            print("Weapons:", {o.nametype for o in self.game.combat_system.weapons})

            settings, initial_state, static_state = self.to_rust_state()

            target_state = cheats_rust.PhysState(
                player=cheats_rust.PlayerState(
                    x=target_x,
                    y=target_y,
                    hx_min=target_x - 16,
                    hx_max=target_x - 16,
                    hy_min=target_y + 16,
                    hy_max=target_y + 16,
                    vx=0,
                    vy=0,
                    vpush=0,
                    direction=cheats_rust.Direction.N,
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
                self.force_movement_keys = []
                for move, shift, state in path:
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

                    self.force_movement_keys.append(moves)

                print("path found", [x[:2] for x in path])
