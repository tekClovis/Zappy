##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## protocol.py
##

from __future__ import annotations
import re
from dataclasses import dataclass
from enum import Enum, auto

# Inventory lines always have "food N" (resource followed by a quantity).
# Look lines may contain "food" as a bare item — no number follows it.
_INVENTORY_RE = re.compile(r'\bfood\s+\d')

RESOURCES = ("food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame")


class ServerMsg(Enum):
    OK = auto()
    KO = auto()
    DEAD = auto()
    ELEVATION_UNDERWAY = auto()


@dataclass(frozen=True)
class LookResponse:
    tiles: tuple[tuple[str, ...], ...]

    def current_tile(self) -> tuple[str, ...]:
        return self.tiles[0] if self.tiles else ()


@dataclass(frozen=True)
class InventoryResponse:
    resources: dict[str, int]


@dataclass(frozen=True)
class MessageEvent:
    direction: int
    text: str


@dataclass(frozen=True)
class EjectEvent:
    direction: int


@dataclass(frozen=True)
class CurrentLevelEvent:
    level: int


# Lines that arrive spontaneously, outside the request/response flow.
_UNSOLICITED_PREFIXES = ("message ", "eject: ", "Current level: ", "dead")


def is_unsolicited(line: str) -> bool:
    return any(line.startswith(p) for p in _UNSOLICITED_PREFIXES)


def parse_response(line: str) -> ServerMsg | LookResponse | InventoryResponse | int | str:
    """Parse a direct response to a pending command."""
    match line:
        case "ok":
            return ServerMsg.OK
        case "ko":
            return ServerMsg.KO
        case "dead":
            return ServerMsg.DEAD
        case "Elevation underway":
            return ServerMsg.ELEVATION_UNDERWAY

    if line.startswith("[") and line.endswith("]"):
        inner = line[1:-1]
        return _parse_inventory(inner) if _INVENTORY_RE.search(inner) else _parse_look(inner)

    try:
        return int(line)
    except ValueError:
        return line


def parse_unsolicited(line: str) -> MessageEvent | EjectEvent | CurrentLevelEvent | ServerMsg:
    """Parse a spontaneous server notification."""
    if line.startswith("message "):
        rest = line[len("message "):]
        k_str, _, text = rest.partition(",")
        return MessageEvent(int(k_str), text)

    if line.startswith("eject: "):
        return EjectEvent(int(line[len("eject: "):]))

    if line.startswith("Current level: "):
        return CurrentLevelEvent(int(line[len("Current level: "):]))

    if line == "dead":
        return ServerMsg.DEAD

    return line


def _parse_look(inner: str) -> LookResponse:
    tiles = tuple(
        tuple(item for item in tile_str.strip().split() if item)
        for tile_str in inner.split(",")
    )
    return LookResponse(tiles)


def _parse_inventory(inner: str) -> InventoryResponse:
    resources: dict[str, int] = {}
    for part in inner.split(","):
        part = part.strip()
        if not part:
            continue
        name, _, qty = part.rpartition(" ")
        resources[name.strip()] = int(qty)
    return InventoryResponse(resources)
