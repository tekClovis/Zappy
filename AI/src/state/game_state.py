##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## game_state.py
##

from __future__ import annotations
from dataclasses import dataclass, field
from ..client.connection import ZappyConnection
import time

RESOURCES = ("food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame")

# Stones + player count required to start each elevation ritual.
ELEVATION_REQUIREMENTS: dict[int, dict[str, int]] = {
    1: {"players": 1, "linemate": 1},
    2: {"players": 2, "linemate": 1, "deraumere": 1, "sibur": 1},
    3: {"players": 2, "linemate": 2, "sibur": 1, "phiras": 2},
    4: {"players": 4, "linemate": 1, "deraumere": 1, "sibur": 2, "phiras": 1},
    5: {"players": 4, "linemate": 1, "deraumere": 2, "sibur": 1, "mendiane": 3},
    6: {"players": 6, "linemate": 1, "deraumere": 2, "sibur": 3, "phiras": 1},
    7: {"players": 6, "linemate": 2, "deraumere": 2, "sibur": 2, "mendiane": 2, "phiras": 2, "thystame": 1},
}

ELEVATION_REQUIREMENTS_MAX: dict[int, dict[str, int]] = {
    1: {"players": 1, "linemate": 1},
    2: {"players": 2, "linemate": 3, "deraumere": 1, "sibur": 2, "phiras": 2},
    3: {"players": 2, "linemate": 2, "sibur": 1, "phiras": 2},
    4: {"players": 4, "linemate": 2, "deraumere": 3, "sibur": 3, "mendiane": 3, "phiras": 1},
    5: {"players": 4, "linemate": 1, "deraumere": 2, "sibur": 1, "mendiane": 3},
    6: {"players": 6, "linemate": 3, "deraumere": 4, "sibur": 5, "mendiane": 2, "phiras": 3, "thystame": 1},
    7: {"players": 6, "linemate": 2, "deraumere": 2, "sibur": 2, "mendiane": 2, "phiras": 2, "thystame": 1},
}

SAME_PLAYERS_ELEVATION_REQUIREMENTS: dict[int, dict[str, int]] = {
    1: {"linemate": 1},
    2: {"linemate": 3, "deraumere": 1, "sibur": 2, "phiras": 2},
    4: {"linemate": 2, "deraumere": 3, "sibur": 3, "phiras": 1, "mendiane": 3},
    6: {"linemate": 3, "deraumere": 4, "sibur": 5, "phiras": 3, "mendiane": 2, "thystame": 1}
}

LEVEL_3_TO_8_REQUIREMENTS = {
    "linemate": 9,
    "deraumere": 8,
    "sibur": 10,
    "mendiane": 5,
    "phiras": 6,
    "thystame": 1
}


@dataclass
class GameState:
    world_x: int
    world_y: int
    client_num: int
    level: int = 1
    inventory: dict[str, int] = field(
        default_factory=lambda: {r: 0 for r in RESOURCES}
    )
    vision: tuple[tuple[str, ...], ...] = field(default_factory=tuple)
    alive: bool = True
    freq: float = 0.0
    slots_free: int = 0

    def has_resources_for_elevation(self) -> bool:
        """True if the inventory alone satisfies the stone requirements for the next level."""
        req = ELEVATION_REQUIREMENTS_MAX.get(self.level, {})
        # req = SAME_PLAYERS_ELEVATION_REQUIREMENTS.get(ELEVATION_REQUIREMENTS.get(self.level, {}).get("players", 2), {})
        return all(
            self.inventory.get(r, 0) >= qty
            for r, qty in req.items()
            if r != "players"
        )

    def has_resources_for_level_3_to_8(self):
        for resource, nb in self.inventory.resources.items():
            if resource in list(LEVEL_3_TO_8_REQUIREMENTS.keys()) and nb >= LEVEL_3_TO_8_REQUIREMENTS[resource]:
                return True
        return False

    def players_needed_for_elevation(self) -> int:
        return ELEVATION_REQUIREMENTS_MAX.get(self.level, {}).get("players", 1)

    def resources_missing_for_elevation(self) -> dict[str, int]:
        """Returns {resource: missing_qty} for the current elevation."""
        req = ELEVATION_REQUIREMENTS.get(self.level, {})
        # req = SAME_PLAYERS_ELEVATION_REQUIREMENTS.get(ELEVATION_REQUIREMENTS.get(self.level, {}).get("players", 2), {})
        return {
            r: max(0, qty - self.inventory.get(r, 0))
            for r, qty in req.items()
            if r != "players" and qty > self.inventory.get(r, 0)
        }
    
    def resources_missing_for_level_3_to_8(self):
        return {
            r: LEVEL_3_TO_8_REQUIREMENTS[r] - self.inventory[r]
            for r, nb in self.inventory.items()
            if r in list(LEVEL_3_TO_8_REQUIREMENTS.keys())
            and self.inventory[r] < LEVEL_3_TO_8_REQUIREMENTS[r]
        }

    def inventory_covers(self, requirements: dict[str, int]) -> bool:
        """Generic check: does our inventory satisfy an arbitrary stone requirement dict?
 
        Used by SUPPORT players to check readiness against the CHAMPION's
        announced level requirements, not necessarily our own level.
        """
        return all(
            self.inventory.get(r, 0) >= qty
            for r, qty in requirements.items()
            if r != "players"
        )

    async def get_freq(self, conn: ZappyConnection) -> None:
        start = time.time()
        await conn.send("Inventory")
        end = time.time()
        t = end - start
        self.freq = 1/t