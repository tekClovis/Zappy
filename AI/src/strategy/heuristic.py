##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## heuristic.py
##

from __future__ import annotations
import asyncio
import os
import random
import time

_PID = os.getpid()

from ..client.connection import ZappyConnection
from ..client.protocol import CurrentLevelEvent, InventoryResponse, LookResponse, MessageEvent, ServerMsg
from ..state.game_state import ELEVATION_REQUIREMENTS, ELEVATION_REQUIREMENTS_MAX, GameState
from .base import Strategy

_FOOD_CRITICAL = 20
_FOOD_LOW = 40
_SCAN_INTERVAL = 2

_FOLLOWER_WAIT_TIMEOUT_TICKS = 6
_DIR0_GRACE_TICKS = 10

class Role:
    SOLO = "SOLO"
    LEADER = "LEADER"
    FOLLOWER = "FOLLOWER"

class Movement:
    RUN = "RUN"
    STOP = "STOP"

class HeuristicStrategy(Strategy):
    def __init__(self) -> None:
        self._cpt = 0
        self._steps = 0
        self._level = 1
        self._elevation_done = asyncio.Event()
        self._leader_response = asyncio.Event()
        self._incant_direction = None
        self._role = Role.SOLO
        self._movement = Movement.RUN
        self._just_broadcasted = False
        self._wait_for_broadcast = True
        self._food_limit = _FOOD_LOW
        self._remonter_max_food = False

        # LEADER
        self._tell_pos = False
        self._setted_down_ressources = False
        self._reincant = False

        # FOLLOWER
        self._last_incant_update_tick = -10**9

    def handle_event(self, event) -> None:
        if isinstance(event, CurrentLevelEvent):
            self._level = event.level
            self._role = Role.SOLO
            self._incant_direction = None
            self._elevation_done.set()

        elif isinstance(event, MessageEvent) and event.text.startswith("INCANT_") and self._role != Role.LEADER:
            level_part = event.text[len("INCANT_"):len("INCANT_") + 1]
            if not level_part.isdigit() or int(level_part) != self._level:
                print(f"LEVEL PART PAS BON (incant) ({level_part})")
                return

            print(f"[BROADCAST] {event.text} | DIR: {event.direction}")
            self._role = Role.FOLLOWER
            self._incant_direction = event.direction
            self._last_incant_update_tick = self._cpt

            self._movement = Movement.STOP if self._incant_direction == 0 else Movement.RUN
            self._leader_response.set()
            self._wait_for_broadcast = False

        elif isinstance(event, MessageEvent) and event.text.startswith("SUCCESS_INCANT_") and self._role != Role.LEADER:
            level_part = event.text[len("SUCCESS_INCANT_"):]
            if not level_part.isdigit() or int(level_part) != self._level:
                print(f"LEVEL PART PAS BON (success) ({level_part})")
                return
            print(f"[BROADCAST] {event.text}")
            self._role = Role.SOLO
            self._movement = Movement.RUN
            self._incant_direction = None
            self._leader_response.set()

    async def tick(self, state: GameState, conn: ZappyConnection) -> None:
        self._cpt += 1
        await self._refresh(state, conn)
        food = state.inventory.get("food", 0)
        if state.level == 1 and state.has_resources_for_elevation():
            await self._attempt_elevation(state, conn)
            return
        if self._reincant and state.has_resources_for_elevation():
            self._reincant = False
            await self._leader_loop(state, conn)
            return
        if self._incant_direction != 0:
            self._movement = Movement.RUN
            print(f"DIRECTION: {self._incant_direction} | MOVE: {"STOP" if self._movement == Movement.STOP else "RUN"} (in != 0)")

        if self._incant_direction is not None:
            dir_age = self._cpt - self._last_incant_update_tick
            if dir_age > _FOLLOWER_WAIT_TIMEOUT_TICKS:
                self._incant_direction = None
                if self._role == Role.FOLLOWER:
                    self._role = Role.SOLO
                self._movement = Movement.RUN
                self._wait_for_broadcast = False

            elif self._incant_direction == 0:
                if dir_age <= _DIR0_GRACE_TICKS and food >= _FOOD_CRITICAL:
                    self._movement = Movement.STOP
                    return
                self._movement = Movement.RUN
                if food < _FOOD_CRITICAL:
                    self._remonter_max_food = True
                    await self._seek(state, conn, "food")
                    return




        if self._incant_direction != 0:
            await self._collect_current_tile(state, conn)

        print(f"{self._role} -> {self._movement} | food: {food} | dir: {self._incant_direction if self._incant_direction is not None else "None"} | wait for br: {"True" if self._wait_for_broadcast else "False"}")

        if food < _FOOD_CRITICAL:
            if not (self._role == Role.FOLLOWER and food > round(_FOOD_CRITICAL / 2)):
                self._remonter_max_food = True
                if self._role == Role.LEADER:
                    self._role = Role.SOLO
                self._movement = Movement.RUN
                print("BEFORE SEEK IN CRITICAL")
                await self._seek(state, conn, "food")
                return
        print("AFTER CRITICAL")

        if food < _FOOD_LOW and (self._role == Role.SOLO or self._remonter_max_food):
            self._movement = Movement.RUN
            print("BEFORE SEEK IN LOW")
            await self._seek(state, conn, "food")
            return
        else:
            self._remonter_max_food = False
        print("AFTER LOW")

        # LEADER
        if self._role != Role.FOLLOWER and (state.has_resources_for_elevation() or self._setted_down_ressources):
            print("IN IF LEADER")
            self._role = Role.LEADER
            if state.level == 1:
                await self._attempt_elevation(state, conn)
                return
            if self._cpt % 4 == 0:
                await conn.send(f"Broadcast INCANT_{state.level}_{_PID}")

            self._movement = Movement.STOP
            await self._leader_loop(state, conn)
            return
        print("AFTER LEADER")

        if self._role == Role.FOLLOWER:
            dir_age = self._cpt - self._last_incant_update_tick

            if self._wait_for_broadcast and self._incant_direction is not None:
                if dir_age <= _FOLLOWER_WAIT_TIMEOUT_TICKS:
                    return
                self._wait_for_broadcast = False

            if self._incant_direction is not None and self._incant_direction > 0:
                await _step_toward_direction(self._incant_direction, conn, self._movement)
                self._wait_for_broadcast = True
                return

            if self._incant_direction == 0:
                if dir_age <= _DIR0_GRACE_TICKS and food >= _FOOD_CRITICAL:
                    self._movement = Movement.STOP
                    return
                self._incant_direction = None
                self._role = Role.SOLO
                self._movement = Movement.RUN
        print("AFTER FOLLOWER")

        await self._explore(conn)
        print("AFTER EXPLORE")


    
    async def _leader_loop(self, state: GameState, conn: ZappyConnection):
        needed = state.players_needed_for_elevation()

        look = await conn.send("Look")
        if isinstance(look, LookResponse):
            state.vision = look.tiles
        
        players_on_tile = state.vision[0].count("player") if state.vision else 0

        if players_on_tile >= needed:
            await conn.send(f"Broadcast INCANT_{state.level}_{_PID}")
            success = await self._attempt_elevation(state, conn)
            self._movement = Movement.RUN
            self._role = Role.SOLO
            await conn.send(f"Broadcast SUCCESS_INCANT_{state.level}")
            if ELEVATION_REQUIREMENTS_MAX.get(state.level, {}).get("players", 1) == ELEVATION_REQUIREMENTS_MAX.get(state.level - 1, {}).get("players", 1):
                self._reincant = True
                self._movement = Movement.STOP
                self._role = Role.LEADER
            self._setted_down_ressources = False

    async def _attempt_elevation(self, state: GameState, conn: ZappyConnection) -> bool:
        req = ELEVATION_REQUIREMENTS.get(state.level, {})

        for resource, qty in req.items():
            if resource == "players":
                continue
            for _ in range(qty):
                await conn.send(f"Set {resource}")
                print(f"{resource} posé")

        self._setted_down_ressources = True


        look = await conn.send("Look")
        if isinstance(look, LookResponse):
            state.vision = look.tiles
        
        print(f"[LEADER] vision[0]={state.vision[0] if state.vision else []}")
        print(f"COMPLETE VISION: {state.vision}")

        self._elevation_done.clear()
        result = await conn.send("Incantation")
        print(f"RESULT TRY INCANTATION: {result}")

        if result is ServerMsg.ELEVATION_UNDERWAY:
            try:
                await asyncio.wait_for(self._elevation_done.wait(), timeout=float(300/state.freq + 10))
                print(f"[LEADER] Incantation réussie ! Nouveau level: {self._level}")
                return True
            except asyncio.TimeoutError:
                print("[LEADER] Incantation timeout")
                return False
            finally:
                self._elevation_done.clear()
        else:
            print("[LEADER] Incantation ko")
            look = await conn.send("Look")
            if isinstance(look, LookResponse):
                state.vision = look.tiles
            for item in (state.vision[0] if state.vision else ()):
                if item != "player":
                    await conn.send(f"Take {item}")
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
            print("AFTER AWAIT INVENTORY")
            if isinstance(inv, InventoryResponse):
                state.inventory.update(inv.resources)
                print(f"Inventory OK: {state.inventory}")
                break
            else:
                tries += 1
                await asyncio.sleep(0.1)
        
        if tries == max_attempts:
            state.inventory["food"] = 1
        
        look = await conn.send("Look")
        print("AFTER AWAIT LOOK")
        if isinstance(look, LookResponse):
            state.vision = look.tiles

    async def _collect_current_tile(self, state: GameState, conn: ZappyConnection) -> None:
        if self._role == Role.SOLO:
            missing = state.resources_missing_for_elevation()
        for item in (state.vision[0] if state.vision else ()):
            if item == "food" and self._remonter_max_food:
                await conn.send(f"Take {item}")
            elif self._role != Role.SOLO:
                continue
            elif item in missing and missing[item] > 0:
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


def _best_tile_in_vision(vision, resource: str, prefer_density: bool = False):
    best_idx = None
    best_score = float("-inf")

    for idx, tile in enumerate(vision):
        qty = sum(1 for it in tile if it == resource)
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
    elif col > 0:
        await conn.send("Right")
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