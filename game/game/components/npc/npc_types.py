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

from .banker_npc import BankerNPC
from .music_npc import RaccoonMusicNpc
from .npc import BarkNpc
from .password_npc import SnakeNpc
from .password_npc_green import PasswordNpcGreen

NPC_TYPES = {
    "bark_npc": BarkNpc,
    "snake_npc": SnakeNpc,
    "banker_npc": BankerNPC,
    "password_npc_green": PasswordNpcGreen,
    "raccoon_music_npc": RaccoonMusicNpc,
}
