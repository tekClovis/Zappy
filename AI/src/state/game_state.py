##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## game_state.py
##

from __future__ import annotations
from ..client.connection import ZappyConnection
import time

RESOURCES = ("food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame")

ELEVATION_REQUIREMENTS = {
    1: {"players": 1, "linemate": 1},
    2: {"players": 2, "linemate": 1, "deraumere": 1, "sibur": 1},
    3: {"players": 2, "linemate": 2, "sibur": 1, "phiras": 2},
    4: {"players": 4, "linemate": 1, "deraumere": 1, "sibur": 2, "phiras": 1},
    5: {"players": 4, "linemate": 1, "deraumere": 2, "sibur": 1, "mendiane": 3},
    6: {"players": 6, "linemate": 1, "deraumere": 2, "sibur": 3, "phiras": 1},
    7: {"players": 6, "linemate": 2, "deraumere": 2, "sibur": 2, "mendiane": 2, "phiras": 2, "thystame": 1},
}

LEVEL_3_TO_8_REQUIREMENTS = {
    "linemate": 9,
    "deraumere": 8,
    "sibur": 10,
    "mendiane": 5,
    "phiras": 6,
    "thystame": 1
}

class GameState:
    def __init__(self, world_x, world_y, client_num):
        self.world_x = world_x
        self.world_y = world_y
        self.client_num = client_num
        self.level = 1
        self.inventory = {r: 0 for r in RESOURCES}
        self.vision = tuple()
        self.alive = True
        self.freq = 0.0

    def has_resources_for_elevation(self) -> bool:
        for resource, nb in self.inventory.items():
            if resource in list(ELEVATION_REQUIREMENTS.get(self.level, {}).keys()) and not (nb >= ELEVATION_REQUIREMENTS.get(self.level, {})[resource]):
                return False
        return True

    def has_resources_for_level_3_to_8(self):
        for resource, nb in self.inventory.items():
            if resource in list(LEVEL_3_TO_8_REQUIREMENTS.keys()) and not (nb >= LEVEL_3_TO_8_REQUIREMENTS[resource]):
                return False
        return True

    def players_needed_for_elevation(self) -> int:
        return ELEVATION_REQUIREMENTS.get(self.level, {}).get("players", 1)

    def resources_missing_for_elevation(self) -> dict[str, int]:
        missing = {}
        for resource, nb in ELEVATION_REQUIREMENTS.get(self.level, {}).items():
            if resource == "players":
                continue
            current = self.inventory.get(resource, 0)
            if nb > current:
                missing[resource] = nb - current
        return missing

    def resources_missing_for_level_3_to_8(self) -> dict[str, int]:
        missing = {}
        for resource, nb in LEVEL_3_TO_8_REQUIREMENTS.items():
            current = self.inventory.get(resource, 0)
            if current < nb:
                missing[resource] = nb - current
        return missing

    async def get_freq(self, conn: ZappyConnection) -> None:
        start = time.time()
        await conn.send("Inventory")
        end = time.time()
        t = end - start
        self.freq = 1/t