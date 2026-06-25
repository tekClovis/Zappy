##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## protocol.py
##

from __future__ import annotations
import re
from enum import Enum, auto

RESOURCES = ("food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame")


class ServerMsg(Enum):
    OK = auto()
    KO = auto()
    DEAD = auto()
    ELEVATION_UNDERWAY = auto()


class LookResponse:
    def __init__(self, tiles):
        self.tiles = tiles

    def current_tile(self):
        return self.tiles[0] if self.tiles else ()

class InventoryResponse:
    def __init__(self, resources):
        self.resources = resources

class MessageEvent:
    def __init__(self, direction, text):
        self.direction = direction
        self.text = text

class EjectEvent:
    def __init__(self, direction):
        self.direction = direction

class CurrentLevelEvent:
    def __init__(self, level):
        self.level = level

_UNSOLICITED_PREFIXES = ("message ", "eject: ", "Current level: ", "dead")


def is_unsolicited(line: str) -> bool:
    return any(line.startswith(p) for p in _UNSOLICITED_PREFIXES)


def parse_response(line: str):
    line = line.strip()

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
        return _parse_inventory(inner) if re.search(r'\bfood\s+\d', inner) else _parse_look(inner)

    try:
        return int(line)
    except ValueError:
        return line


def parse_unsolicited(line: str):
    if line.startswith("message "):
        rest = line[len("message "):]
        k_str, _, text = rest.partition(", ")
        return MessageEvent(int(k_str), text)

    if line.startswith("eject: "):
        return EjectEvent(int(line[len("eject: "):]))

    if line.startswith("Current level: "):
        return CurrentLevelEvent(int(line[len("Current level: "):]))

    if line == "dead":
        return ServerMsg.DEAD

    return line


def _parse_look(inner: str) -> LookResponse:
    tiles = tuple(tuple(item for item in tile_str.strip().split() if item) for tile_str in inner.split(","))
    return LookResponse(tiles)


def _parse_inventory(inner: str) -> InventoryResponse:
    resources = {}
    for part in inner.split(","):
        part = part.strip()
        if not part:
            continue
        name, _, qty = part.rpartition(" ")
        resources[name.strip()] = int(qty)
    return InventoryResponse(resources)
