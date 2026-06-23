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
from ..client.protocol import CurrentLevelEvent, MessageEvent
from ..state.game_state import GameState
from .base import Strategy

_FOOD_CRITICAL = 20
_FOOD_LOW = 40

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
        self._level = 1
        self._elevation_done = asyncio.Event()
        self._role = Role.SOLO
        self._movement = Movement.RUN
        self._wait_for_broadcast = True
        self._remonter_max_food = False

        # LEADER
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

            pass

        elif isinstance(event, MessageEvent) and event.text.startswith("SUCCESS_INCANT_") and self._role != Role.LEADER:
            level_part = event.text[len("SUCCESS_INCANT_"):]
            if not level_part.isdigit() or int(level_part) != self._level:
                print(f"LEVEL PART PAS BON (success) ({level_part})")
                return
            pass

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
        pass

    async def _attempt_elevation(self, state: GameState, conn: ZappyConnection):
        pass

    async def _seek(self, state: GameState, conn: ZappyConnection, resource: str):
        pass

    async def _explore(self, conn: ZappyConnection):
        pass

    async def _refresh(self, state: GameState, conn: ZappyConnection):
        pass

    async def _collect_current_tile(self, state: GameState, conn: ZappyConnection):
        pass


async def _step_toward_direction(direction: int, conn: ZappyConnection, move: str) -> None:
    pass