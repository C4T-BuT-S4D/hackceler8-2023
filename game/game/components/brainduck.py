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
import time
from ctypes import c_uint8
from enum import Enum
from typing import Optional
from typing import Tuple

from arcade import key

from components.magic_items import Item
from engine import generics
from engine import hitbox
from engine.bf import Interpreter

TICK_GRANULARITY = 5
VALID_KEYS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 62, 60, 46, 44, 43, 45, 91, 93]


class DuckModes(Enum):
    MODE_RECORDING = "recording"
    MODE_EXECUTING = "executing"


class Brainduck(generics.GenericObject):
    order = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

    def __init__(self, coords, size, name, max_steps=1000, instructions_override=None):
        self.perimeter = [
            hitbox.Point(coords.x, coords.y),
            hitbox.Point(coords.x + size.width, coords.y),
            hitbox.Point(coords.x + size.width, coords.y - size.height),
            hitbox.Point(coords.x, coords.y - size.height),
        ]
        super().__init__(coords, "Brainduck", None, self.perimeter)
        self.blocking = False
        self.instructions_override = instructions_override
        self.name = name
        # secrets.seed(seed)
        self.stopped = False
        self.max_steps = max_steps

        self.interpreter = None

        self.last_press_tics = 0
        self.failed = False
        self.total_steps = 0
        self.instructions = bytearray()
        self.walking_data = None

        self.active = False

        self.tics = 0
        self.recorded_tics = 0
        self.mode = None

        self.start_time = 0

        self.duck_ids = []

        self.pc = 0
        self.item_yielded = False

        self.reset()

    def reset(self):
        self.stopped = False
        self.active = False
        self.pc = 0

        self.last_press_tics = 0
        self.failed = False
        self.total_steps = 0
        self.instructions = bytearray()
        self.mode = DuckModes.MODE_RECORDING

    def add_key(self, pressed_keys, newly_pressed_keys):
        self.recorded_tics += 1
        shift = key.LSHIFT in pressed_keys
        pressed = None

        for k in newly_pressed_keys:
            if k != key.LSHIFT:
                pressed = k
                break

        if pressed is None:
            for k in pressed_keys:
                if k != key.LSHIFT:
                    pressed = k
                    # self.last_press_tics = game_tics
                    break

        if pressed and shift:
            match pressed:
                case key.W:
                    self.instructions.append(43)
                case key.S:
                    self.instructions.append(45)
                case key.D:
                    self.instructions.append(91)
                case key.A:
                    self.instructions.append(93)
        elif pressed:
            match pressed:
                case key.W:
                    self.instructions.append(62)
                case key.S:
                    self.instructions.append(60)
                case key.D:
                    self.instructions.append(46)
                case key.A:
                    self.instructions.append(44)

    def start(self):
        self.start_time = time.time()
        self.mode = DuckModes.MODE_EXECUTING
        self.walking_data = self.bf_to_walk(self.instructions)

        if self.instructions_override is not None:
            self.instructions = self.instructions_override
        logging.info(self.instructions)
        logging.info(f"Walk equivalent: " f"{[chr(i[0]) for i in self.walking_data]}")
        logging.info(f"Recorded for a total of {self.recorded_tics}")
        logging.info(f"Recorded a total of {len(self.instructions)} steps")

        self.interpreter = Interpreter(self.instructions)
        self.interpreter.order = self.order
        logging.info(f"Execution started with order {self.order}")
        self.interpreter.start()

    def stop(self):
        self.stopped = True

    def run_to_print(self) -> Optional[int]:
        if self.stopped:
            return

        gotten = self.interpreter.execute_until_return()
        if gotten is None:
            return None
        logging.debug(f"Got {gotten}")
        if gotten in VALID_KEYS:
            return self.num_to_keys(gotten)
        else:
            logging.critical(f"Invalid key received: {gotten}")

    def bf_to_walk(self, bts):
        return [self.num_to_keys(i) for i in bts]

    @staticmethod
    def num_to_keys(pressed: int) -> Optional[Tuple[int, bool]]:
        match pressed:
            case 1 | 62:
                return key.W, False
            case 2 | 60:
                return key.S, False
            case 3 | 46:
                return key.D, False
            case 4 | 44:
                return key.A, False
            case 5 | 43:
                return key.W, True
            case 6 | 45:
                return key.S, True
            case 7 | 91:
                return key.D, True
            case 8 | 93:
                return key.A, True
            case other:
                None

    def yield_item(self):
        if not self.item_yielded:
            self.item_yielded = True
            return Item(coords=None, name="boots", display_name="Boots", wearable=True)

    def check_order(self):
        if self.duck_ids == self.interpreter.order:
            return self.yield_item()
        logging.info(
            f"Wrong order detected (-want + got): {self.interpreter.order, self.duck_ids}"
        )

    def tick(self, pressed_keys, newly_pressed_keys):
        match self.mode:
            case DuckModes.MODE_RECORDING:
                self.add_key(pressed_keys, newly_pressed_keys)
                logging.debug("Recording")
                return "recording"
            case DuckModes.MODE_EXECUTING:
                logging.debug("Executing")
                return self.run_to_print()
