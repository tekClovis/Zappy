##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## heuristic.py
##

from __future__ import annotations
import asyncio
import os

_PID = os.getpid()

from ..client.connection import ZappyConnection
from ..client.protocol import CurrentLevelEvent, InventoryResponse, LookResponse, MessageEvent, ServerMsg
from ..state.game_state import ELEVATION_REQUIREMENTS, GameState
from .base import Strategy

_FOOD_CRITICAL = 20
_FOOD_LOW = 40
_SCAN_INTERVAL = 2

class Role:
    SOLO = "SOLO"
    LEADER = "LEADER"
    FOLLOWER = "FOLLOWER"

class Movement:
    RUN = "RUN"
    STOP = "STOP"

class HeuristicStrategy(Strategy):
    def __init__(self):
        self._pid = str(_PID)
        self._cpt = 0
        self._steps = 0
        self._level = 1
        self._elevation_done = asyncio.Event()
        self._role = Role.SOLO
        self._movement = Movement.RUN
        self._others = dict()
        self._wait_broadcast = True
        self._pid_leader = ""
        self._food = 0
        self._respond_to_br = False
        self._grind_low_food = False
        self._block_cpt_mate_dir = 0

        # LEADER
        self._leader_can_incant_lvl_8 = False
        self._setted_down_resources = False

        # FOLLOWER
        self._leader_direction = None

        # SOLO
        self._mate_direction = None
        self._start_ask_incant = False
        self._pid_to_help = ""
        self._helper_pid = ""


    def handle_event(self, event) -> None:
        if isinstance(event, CurrentLevelEvent):
            self._respond_to_br = True
            self._level = event.level
            if self._level < 4:
                self._leader_direction = None
            else:
                self._leader_direction = 0
            self._elevation_done.set()
        
        elif isinstance(event, MessageEvent) and event.text.startswith("PLAYER_"): # Broadcast PLAYER_<level>_<pid>
            level_part = event.text[len("PLAYER_"):len("PLAYER_") + 1]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("PLAYER_") + 2:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            if pid_part not in list(self._others.keys()):
                self._respond_to_br = True
            level = int(level_part)
            self._others[pid_part] = level

        elif isinstance(event, MessageEvent) and event.text.startswith("HERE_FOR_HELP_"): # Broadcast HERE_FOR_HELP_<recPID>_<level>_<pid>
            my_pid = event.text[len("HERE_FOR_HELP_"):len("HERE_FOR_HELP_") + len(self._pid)]
            if my_pid != self._pid:
                return
            level_part = event.text[len("HERE_FOR_HELP_") + len(self._pid) + 1:len("HERE_FOR_HELP_") + len(self._pid) + 2]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("HERE_FOR_HELP_") + len(self._pid) + 3:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            self._others[pid_part] = int(level_part)
            if int(level_part) != self._level:
                return
            if self._helper_pid == "":
                self._helper_pid = pid_part

        elif isinstance(event, MessageEvent) and event.text.startswith("HELP_"): # Broadcast HELP_<level>_<pid>
            level_part = event.text[len("HELP_"):len("HELP_") + 1]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("HELP_") + 2:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            self._others[pid_part] = int(level_part)
            if int(level_part) != self._level:
                return
            if self._pid_to_help != "" and pid_part != self._pid_to_help:
                return
            self._pid_to_help = pid_part
            self._mate_direction = event.direction
            self._movement = Movement.STOP if event.direction == 0 else Movement.RUN
            self._wait_broadcast = False
        
        elif isinstance(event, MessageEvent) and event.text.startswith("HELP2_"): # Broadcast HELP_<recPID>_<level>_<pid>
            my_pid = event.text[len("HELP2_"):len("HELP2_") + len(self._pid)]
            if my_pid != self._pid:
                return
            level_part = event.text[len("HELP2_") + len(self._pid) + 1:len("HELP2_") + len(self._pid) + 2]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("HELP2_") + len(self._pid) + 3:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            self._others[pid_part] = int(level_part)
            self._pid_to_help = pid_part
            self._mate_direction = event.direction
            self._movement = Movement.STOP if event.direction == 0 else Movement.RUN
            self._wait_broadcast = False
        
        elif isinstance(event, MessageEvent) and event.text.startswith("SUCCESS_HELP_"): # Broadcast SUCCESS_HELP_<level>_<pid>
            level_part = event.text[len("SUCCESS_HELP_"):len("SUCCESS_HELP_") + 1]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("SUCCESS_HELP_") + 2:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            self._others[pid_part] = int(level_part)
            if pid_part != self._pid_to_help:
                return
            self._pid_to_help = ""
            self._mate_direction = None
            self._movement = Movement.RUN
            self._wait_broadcast = True

        elif isinstance(event, MessageEvent) and event.text.startswith("I_AM_LEADER_") and self._role != Role.LEADER: # Broadcast I_AM_LEADER_<level>_<pid>
            level_part = event.text[len("I_AM_LEADER_"):len("I_AM_LEADER_") + 1]
            if not level_part.isdigit():
                return
            pid_part = event.text[len("I_AM_LEADER_") + 2:]
            if not pid_part.isdigit():
                return
            if pid_part in list(self._others.keys()):
                self._others.pop(pid_part)
            self._others["LEADER"] = int(level_part)
            self._pid_leader = "LEADER"
            if self._level >= 3:
                self._role = Role.FOLLOWER

        elif isinstance(event, MessageEvent) and event.text.startswith("INCANT_") and self._role != Role.LEADER and self._level >= 3: # Broadcast INCANT_<level>_<pid>
            level_part = event.text[len("INCANT_"):len("INCANT_") + 1]
            if not level_part.isdigit():
                self._role = Role.SOLO
                self._movement = Movement.RUN
                self._leader_direction = None
                return
            pid_part = event.text[len("INCANT_") + 2:]
            if not pid_part.isdigit() and pid_part != "LEADER":
                return
            self._others[pid_part] = int(level_part)
            if int(level_part) != self._level or pid_part != self._pid_leader:
                return
            self._leader_direction = event.direction
            self._role = Role.FOLLOWER
            self._movement = Movement.STOP if event.direction == 0 else Movement.RUN
            self._wait_broadcast = False

    async def tick(self, state: GameState, conn: ZappyConnection) -> None:
        if self._cpt == 0 or self._respond_to_br:
            await conn.send(f"Broadcast PLAYER_{self._level}_{self._pid}")
            self._respond_to_br = False
        self._cpt += 1

        await self._refresh(state, conn)
        self._food = state.inventory.get("food", 0)

        print(f"[{self._pid}] {self._role} -> {self._movement} | lvl: {state.level} | food: {self._food} | Ldir: {self._leader_direction} | Mdir: {self._mate_direction} | PID_to_help: {self._pid_to_help} | others: {self._others}")

        if self._leader_direction != 0:
            await self._collect_current_tile(state, conn)


        if self._role == Role.SOLO:
            await self._solo_loop(state, conn)

        elif self._role == Role.FOLLOWER:
            await self._follower_loop(state, conn)

        elif self._role == Role.LEADER:
            await self._leader_loop(state, conn)



    async def _leader_loop_incant(self, state: GameState, conn: ZappyConnection):
        self._leader_can_incant_lvl_8 = True

        look = await conn.send("Look")
        if isinstance(look, LookResponse):
            state.vision = look.tiles

        await conn.send("Take food")

        players_on_tile = state.vision[0].count("player") if state.vision else 0

        await conn.send(f"Broadcast INCANT_{state.level}_{self._pid}")
        if players_on_tile >= 6:
            await self._attempt_elevation(state, conn)
            await asyncio.sleep(1)

    async def _leader_loop_get_resources(self, state: GameState, conn: ZappyConnection):
        if self._grind_low_food:
            if self._food >= _FOOD_LOW:
                self._grind_low_food = False
            await self._seek(state, conn, "food")
            return
        if self._food < _FOOD_CRITICAL:
            self._grind_low_food = True
            self._movement = Movement.RUN
            await self._seek(state, conn, "food")
            return
        if not state.has_resources_for_level_3_to_8():
            await self._explore(conn)
        elif state.has_resources_for_level_3_to_8() and self._food >= _FOOD_LOW:
            self._leader_can_incant_lvl_8 = True
        else:
            self._grind_low_food = True
            self._movement = Movement.RUN
            await self._seek(state, conn, "food")



    async def _leader_loop(self, state: GameState, conn: ZappyConnection):
        if state.has_resources_for_level_3_to_8() or self._leader_can_incant_lvl_8:
            self._movement = Movement.STOP
            await self._leader_loop_incant(state, conn)
        else:
            await self._leader_loop_get_resources(state, conn)

    async def _follower_loop(self, state: GameState, conn: ZappyConnection):
        if self._leader_direction is not None:
            if self._leader_direction == 0:
                if self._food > _FOOD_LOW:
                    await conn.send(f"Set food")
                    self._food -= 1
                if self._food <= _FOOD_CRITICAL:
                    await conn.send("Take food")
                self._movement = Movement.STOP
                return
            if not self._wait_broadcast:
                await _step_toward_direction(self._leader_direction, conn, self._movement)
                self._wait_broadcast = True
                return
        else:
            await self._seek(state, conn, "food")


    async def _solo_loop(self, state: GameState, conn: ZappyConnection):
        if self._mate_direction == 0:
            await conn.send(f"Broadcast HERE_FOR_HELP_{self._pid_to_help}_{state.level}_{self._pid}")
        if self._grind_low_food:
            if self._food >= _FOOD_LOW:
                self._grind_low_food = False
            await self._seek(state, conn, "food")
            return
        if self._mate_direction is not None and self._food >= _FOOD_CRITICAL:
            if not self._wait_broadcast:
                self._block_cpt_mate_dir = 0
                await _step_toward_direction(self._mate_direction, conn, self._movement)
                self._wait_broadcast = True
            else:
                self._block_cpt_mate_dir += 1
                if self._block_cpt_mate_dir >= 50:
                    self._mate_direction = None
                    self._pid_to_help = ""
                    self._movement = Movement.RUN
            return
        elif self._mate_direction is not None and self._food < _FOOD_CRITICAL:
            self._grind_low_food = True
            await self._seek(state, conn, "food")
            return
        if state.has_resources_for_elevation() and self._level == 1:
            await self._attempt_elevation(state, conn)
            return
        if state.has_resources_for_elevation() and (self._food >= _FOOD_LOW or self._start_ask_incant) and all(self._pid > other_pid for other_pid in list(self._others.keys()) if self._others[other_pid] == 2):
            self._start_ask_incant = False if self._food < _FOOD_CRITICAL else True
            if self._level == 2:

                await self._check_players_max_on_map(state, conn)

                look = await conn.send("Look")
                if isinstance(look, LookResponse):
                    state.vision = look.tiles

                players_on_tile = state.vision[0].count("player") if state.vision else 0

                if self._helper_pid == "":
                    await conn.send(f"Broadcast HELP_{state.level}_{self._pid}")
                else:
                    await conn.send(f"Broadcast HELP2_{self._helper_pid}_{state.level}_{self._pid}")
                if players_on_tile == 2:
                    success = await self._attempt_elevation(state, conn)
                    if success:
                        self._start_ask_incant = False
                        self._helper_pid = ""
                        await conn.send(f"Broadcast SUCCESS_HELP_{state.level}_{self._pid}")
                return

        if self._level == 3 and "LEADER" in list(self._others.keys()):
            self._role = Role.FOLLOWER
            return
        if self._level == 3 and "LEADER" not in list(self._others.keys()) and all(self._pid > other_pid for other_pid in list(self._others.keys()) if self._others[other_pid] == 3):
            self._pid = "LEADER"
            await conn.send(f"Broadcast I_AM_LEADER_{self._level}_{_PID}")
            self._role = Role.LEADER
            return
        
        if self._food <= _FOOD_CRITICAL and self._level != 1:
            self._grind_low_food = True
            return

        if state.inventory.get("linemate", 0) == 0 and self._food >= _FOOD_CRITICAL:
            await self._seek(state, conn, "linemate")
        elif state.inventory.get("deraumere", 0) == 0 and self._food >= _FOOD_CRITICAL:
            await self._seek(state, conn, "deraumere")
        elif state.inventory.get("sibur", 0) == 0 and self._food >= _FOOD_CRITICAL:
            await self._seek(state, conn, "sibur")
        else:
            await self._seek(state, conn, "food")
    
    async def _check_players_max_on_map(self, state: GameState, conn: ZappyConnection):
        players_on_map = len(self._others) + 1

        if players_on_map < 6:
            player_can_connect_str = await conn.send("Connect_nbr")
            player_can_connect = int(player_can_connect_str) if player_can_connect_str != "ko" else 0
            diff = 6 - (players_on_map + player_can_connect) if (players_on_map + player_can_connect) <= 6 else 0

            for _ in range(diff):
                await conn.send("Fork")

    async def _attempt_elevation(self, state: GameState, conn: ZappyConnection) -> bool:
        req = ELEVATION_REQUIREMENTS.get(state.level, {})

        for resource, qty in req.items():
            if resource == "players":
                continue
            for _ in range(qty):
                await conn.send(f"Set {resource}")
        self._setted_down_resources = True


        look = await conn.send("Look")
        if isinstance(look, LookResponse):
            state.vision = look.tiles

        self._elevation_done.clear()
        result = await conn.send("Incantation")

        if result is ServerMsg.ELEVATION_UNDERWAY:
            try:
                await asyncio.wait_for(self._elevation_done.wait(), timeout=float(300/state.freq + 10))
                await conn.send(f"Broadcast PLAYER_{self._level}_{self._pid}")
                self._setted_down_resources = False
                return True
            except asyncio.TimeoutError:
                self._setted_down_resources = False
                return False
            finally:
                self._elevation_done.clear()
                self._setted_down_resources = False
        else:
            look = await conn.send("Look")
            if isinstance(look, LookResponse):
                state.vision = look.tiles
            for item in (state.vision[0] if state.vision else ()):
                if item != "player":
                    await conn.send(f"Take {item}")
            self._setted_down_resources = False
            return False

    async def _seek(self, state: GameState, conn: ZappyConnection, resource: str) -> None:
        if self._movement == Movement.STOP:
            return

        current_tile = state.vision[0] if state.vision else ()
        if resource in current_tile:
            await conn.send(f"Take {resource}")
            return

        food = state.inventory.get("food", 0)
        is_critical_food = resource == "food" and food < _FOOD_CRITICAL
        target_idx = _best_tile_in_vision(state.vision, resource, prefer_density=is_critical_food)

        if target_idx is not None:
            await _move_toward(target_idx, conn, self._movement)
            return

        await self._explore(conn)

    async def _explore(self, conn: ZappyConnection) -> None:
        if self._movement == Movement.STOP:
            return
        self._steps += 1
        if self._steps % 3 == 0:
            await conn.send("Left")
        elif self._steps % _SCAN_INTERVAL == 0:
            await conn.send("Right")
        await conn.send("Forward")

    async def _refresh(self, state: GameState, conn: ZappyConnection) -> None:
        max_attempts = 3
        tries = 0
        for attempt in range(max_attempts):
            inv = await conn.send("Inventory")
            if isinstance(inv, InventoryResponse):
                inv.resources
                state.inventory.update(inv.resources)
                break
            else:
                tries += 1
                await asyncio.sleep(0.1)
        
        if tries == max_attempts:
            state.inventory["food"] = 1
        
        look = await conn.send("Look")
        if isinstance(look, LookResponse):
            state.vision = look.tiles

    async def _collect_current_tile(self, state: GameState, conn: ZappyConnection) -> None:
        if self._leader_direction == 0:
            return
        if self._role == Role.SOLO:
            missing = state.resources_missing_for_elevation()
        elif self._role == Role.LEADER:
            missing = state.resources_missing_for_level_3_to_8()

        food = state.inventory.get("food", 0)
        for item in (state.vision[0] if state.vision else ()):
            if item == "food":
                if self._role != Role.LEADER or (food < _FOOD_CRITICAL and not self._setted_down_resources):
                    await conn.send(f"Take {item}")
                continue
            if self._role != Role.FOLLOWER:
                if item in list(missing.keys()) and missing[item] > 0:
                    await conn.send(f"Take {item}")
                    missing[item] -= 1

def _tile_col_offset(tile_idx: int) -> int:
    if tile_idx == 0:
        return 0
    k = int(tile_idx ** 0.5)
    if k * k + 2 * k < tile_idx:
        k += 1
    return tile_idx - k * k - k


def _tile_row_col(tile_idx: int):
    if tile_idx == 0:
        return 0, 0

    row = 1
    start = 1
    while True:
        width = 2 * row + 1
        end = start + width - 1
        if start <= tile_idx <= end:
            col = (tile_idx - start) - row
            return row, col
        start = end + 1
        row += 1


def _best_tile_in_vision(vision, resource: str, prefer_density: bool = False) -> int | None:
    best_idx: int | None = None
    best_score = float("-inf")

    for idx, tile in enumerate(vision):
        qty = sum(1 for item in tile if item == resource)
        if qty == 0:
            continue

        row, col = _tile_row_col(idx)
        density_weight = 7.0 if prefer_density else 4.0
        score = (qty * density_weight) - (row * 1.6) - (abs(col) * 0.55)

        if col == 0:
            score += 0.4

        if score > best_score:
            best_score = score
            best_idx = idx

    return best_idx

async def _move_toward(tile_idx: int, conn: ZappyConnection, move: str) -> None:
    if move == Movement.STOP:
        return
    col = _tile_col_offset(tile_idx)
    if col < 0:
        await conn.send("Left")
        await conn.send("Forward")
        await conn.send("Right")
    elif col > 0:
        await conn.send("Right")
        await conn.send("Forward")
        await conn.send("Left")
    else:
        await conn.send("Forward")


async def _step_toward_direction(direction: int, conn: ZappyConnection, move: str) -> None:
    if move == Movement.STOP:
        return
    match direction:
        case 0:
            pass
        case 1:
            await conn.send("Forward")
        case 2:
            await conn.send("Left")
            await conn.send("Forward")
        case 3:
            await conn.send("Left")
            await conn.send("Forward")
        case 4:
            await conn.send("Left")
            await conn.send("Forward")
        case 5:
            await conn.send("Left")
            await conn.send("Left")
            await conn.send("Forward")
        case 6:
            await conn.send("Right")
            await conn.send("Forward")
        case 7:
            await conn.send("Right")
            await conn.send("Forward")
        case 8:
            await conn.send("Right")
            await conn.send("Forward")