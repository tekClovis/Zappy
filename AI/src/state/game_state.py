##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## game_state.py
##

from dataclasses import dataclass, field

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

    def has_resources_for_elevation(self) -> bool:
        """True if the inventory alone satisfies the stone requirements for the next level."""
        req = ELEVATION_REQUIREMENTS.get(self.level, {})
        return all(
            self.inventory.get(r, 0) >= qty
            for r, qty in req.items()
            if r != "players"
        )

    def players_needed_for_elevation(self) -> int:
        return ELEVATION_REQUIREMENTS.get(self.level, {}).get("players", 1)

    def resources_missing_for_elevation(self) -> dict[str, int]:
        """Returns {resource: missing_qty} for the current elevation."""
        req = ELEVATION_REQUIREMENTS.get(self.level, {})
        return {
            r: max(0, qty - self.inventory.get(r, 0))
            for r, qty in req.items()
            if r != "players" and qty > self.inventory.get(r, 0)
        }
