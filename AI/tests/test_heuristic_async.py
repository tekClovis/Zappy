##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## test_heuristic_async.py — tests for async strategy methods using a mock connection
##

import asyncio
import pytest
from unittest.mock import AsyncMock, MagicMock

from src.client.protocol import ServerMsg, LookResponse, CurrentLevelEvent
from src.state.game_state import GameState
from src.strategy.heuristic import HeuristicStrategy, Role, Movement


def make_vision(*tile_contents):
    return tuple(tuple(item for item in t.split() if item) for t in tile_contents)


def make_inventory(**kwargs):
    base = {"food": 10, "linemate": 0, "deraumere": 0, "sibur": 0, "mendiane": 0, "phiras": 0, "thystame": 0}
    base.update(kwargs)
    return base


def make_state(level=1, inventory=None, vision=None, freq=100.0):
    state = GameState(world_x=10, world_y=10, client_num=1)
    state.level = level
    state.freq = freq
    if inventory:
        state.inventory.update(inventory)
    if vision is not None:
        state.vision = vision
    return state


def make_conn(responses=None):
    """Return a mock ZappyConnection whose send() consumes a list of preset responses."""
    conn = MagicMock()
    response_iter = iter(responses or [])

    async def _send(cmd):
        try:
            return next(response_iter)
        except StopIteration:
            return ServerMsg.OK

    conn.send = AsyncMock(side_effect=_send)
    return conn


class TestAttemptElevation:
    @pytest.mark.asyncio
    async def test_level1_sets_linemate_then_incants(self):
        s = HeuristicStrategy()
        state = make_state(level=1, inventory=make_inventory(linemate=1))
        state.vision = make_vision("player linemate")
        conn = make_conn()

        # Provide responses: Set→ok, Look→tiles, Incantation→ko (fail fast)
        look_resp = LookResponse(make_vision("player"))
        conn.send = AsyncMock(side_effect=[
            ServerMsg.OK,           # Set linemate
            look_resp,              # Look
            ServerMsg.KO,           # Incantation → fail
            look_resp,              # Look (recovery)
        ])

        result = await s._attempt_elevation(state, conn)
        assert result is False

        sent_commands = [c.args[0] for c in conn.send.call_args_list]
        assert "Set linemate" in sent_commands
        assert "Incantation" in sent_commands

    @pytest.mark.asyncio
    async def test_level1_success_returns_true(self):
        s = HeuristicStrategy()
        state = make_state(level=1, inventory=make_inventory(linemate=1), freq=100.0)
        state.vision = make_vision("player")
        look_resp = LookResponse(make_vision("player"))

        async def side_effect(cmd):
            if cmd == "Set linemate":
                return ServerMsg.OK
            if cmd == "Look":
                return look_resp
            if cmd == "Incantation":
                s.handle_event(CurrentLevelEvent(2))
                return ServerMsg.ELEVATION_UNDERWAY
            return ServerMsg.OK

        conn = MagicMock()
        conn.send = AsyncMock(side_effect=side_effect)

        result = await s._attempt_elevation(state, conn)
        assert result is True

    @pytest.mark.asyncio
    async def test_level2_sets_correct_resources(self):
        s = HeuristicStrategy()
        state = make_state(level=2, inventory=make_inventory(linemate=1, deraumere=1, sibur=1), freq=100.0)
        state.vision = make_vision("player")
        look_resp = LookResponse(make_vision("player"))

        sent = []

        async def side_effect(cmd):
            sent.append(cmd)
            if cmd == "Incantation":
                s.handle_event(CurrentLevelEvent(3))
                return ServerMsg.ELEVATION_UNDERWAY
            if cmd == "Look":
                return look_resp
            return ServerMsg.OK

        conn = MagicMock()
        conn.send = AsyncMock(side_effect=side_effect)

        await s._attempt_elevation(state, conn)

        assert "Set linemate" in sent
        assert "Set deraumere" in sent
        assert "Set sibur" in sent

    @pytest.mark.asyncio
    async def test_failed_elevation_picks_up_resources(self):
        s = HeuristicStrategy()
        state = make_state(level=1, inventory=make_inventory(linemate=1), freq=100.0)
        look_with_linemate = LookResponse(make_vision("player linemate"))

        async def side_effect(cmd):
            if cmd == "Look":
                return look_with_linemate
            if cmd == "Incantation":
                return ServerMsg.KO
            return ServerMsg.OK

        conn = MagicMock()
        conn.send = AsyncMock(side_effect=side_effect)

        result = await s._attempt_elevation(state, conn)
        assert result is False

        sent = [c.args[0] for c in conn.send.call_args_list]
        # After failed incantation, should try to take back linemate
        assert "Take linemate" in sent

    @pytest.mark.asyncio
    async def test_setted_down_resources_cleared_on_success(self):
        s = HeuristicStrategy()
        state = make_state(level=1, inventory=make_inventory(linemate=1), freq=100.0)
        look_resp = LookResponse(make_vision("player"))

        async def side_effect(cmd):
            if cmd == "Incantation":
                s.handle_event(CurrentLevelEvent(2))
                return ServerMsg.ELEVATION_UNDERWAY
            if cmd == "Look":
                return look_resp
            return ServerMsg.OK

        conn = MagicMock()
        conn.send = AsyncMock(side_effect=side_effect)

        await s._attempt_elevation(state, conn)
        assert s._setted_down_resources is False

    @pytest.mark.asyncio
    async def test_setted_down_resources_cleared_on_failure(self):
        s = HeuristicStrategy()
        state = make_state(level=1, inventory=make_inventory(linemate=1), freq=100.0)
        look_resp = LookResponse(make_vision("player"))

        async def side_effect(cmd):
            if cmd == "Look":
                return look_resp
            if cmd == "Incantation":
                return ServerMsg.KO
            return ServerMsg.OK

        conn = MagicMock()
        conn.send = AsyncMock(side_effect=side_effect)

        await s._attempt_elevation(state, conn)
        assert s._setted_down_resources is False


class TestCollectCurrentTile:
    @pytest.mark.asyncio
    async def test_takes_missing_resource_from_tile(self):
        s = HeuristicStrategy()
        s._role = Role.SOLO
        s._leader_direction = None  # not 0 → proceeds
        state = make_state(level=1, inventory=make_inventory(linemate=0))
        state.vision = make_vision("player linemate")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Take linemate" in sent

    @pytest.mark.asyncio
    async def test_does_not_take_non_missing_resource(self):
        s = HeuristicStrategy()
        s._role = Role.SOLO
        s._leader_direction = None
        # Already has linemate, none missing
        state = make_state(level=1, inventory=make_inventory(linemate=1))
        state.vision = make_vision("player linemate")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Take linemate" not in sent

    @pytest.mark.asyncio
    async def test_returns_early_when_leader_direction_is_0(self):
        s = HeuristicStrategy()
        s._role = Role.SOLO
        s._leader_direction = 0
        state = make_state(level=1, inventory=make_inventory(linemate=0))
        state.vision = make_vision("player linemate")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        conn.send.assert_not_called()

    @pytest.mark.asyncio
    async def test_follower_takes_food_but_not_stones(self):
        s = HeuristicStrategy()
        s._role = Role.FOLLOWER
        s._leader_direction = 2
        state = make_state(level=3, inventory=make_inventory(food=5))
        state.vision = make_vision("player food linemate")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Take food" in sent
        assert "Take linemate" not in sent

    @pytest.mark.asyncio
    async def test_leader_does_not_take_food_when_above_critical(self):
        s = HeuristicStrategy()
        s._role = Role.LEADER
        s._leader_direction = 2
        s._setted_down_resources = False
        state = make_state(level=5, inventory=make_inventory(food=50))
        state.vision = make_vision("player food")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Take food" not in sent

    @pytest.mark.asyncio
    async def test_respects_quantity_limit_per_resource(self):
        s = HeuristicStrategy()
        s._role = Role.SOLO
        s._leader_direction = None
        # Level 1 needs 1 linemate; tile has 3
        state = make_state(level=1, inventory=make_inventory(linemate=0))
        state.vision = make_vision("player linemate linemate linemate")

        conn = make_conn()
        await s._collect_current_tile(state, conn)

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert sent.count("Take linemate") == 1


class TestSeek:
    @pytest.mark.asyncio
    async def test_takes_resource_on_current_tile(self):
        s = HeuristicStrategy()
        s._movement = Movement.RUN
        state = make_state(level=1, inventory=make_inventory(food=5))
        state.vision = make_vision("player food", "")

        conn = make_conn()
        await s._seek(state, conn, "food")

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Take food" in sent

    @pytest.mark.asyncio
    async def test_moves_toward_resource_on_other_tile(self):
        s = HeuristicStrategy()
        s._movement = Movement.RUN
        state = make_state(level=1, inventory=make_inventory(food=5))
        # food is on tile 2 (directly ahead at row 1 center)
        state.vision = make_vision("player", "", "food", "")

        conn = make_conn()
        await s._seek(state, conn, "food")

        sent = [c.args[0] for c in conn.send.call_args_list]
        assert "Forward" in sent

    @pytest.mark.asyncio
    async def test_explores_when_resource_not_in_vision(self):
        s = HeuristicStrategy()
        s._movement = Movement.RUN
        state = make_state(level=1, inventory=make_inventory(food=5))
        state.vision = make_vision("player", "", "")

        conn = make_conn()
        await s._seek(state, conn, "linemate")

        conn.send.assert_called()

    @pytest.mark.asyncio
    async def test_does_nothing_when_stopped(self):
        s = HeuristicStrategy()
        s._movement = Movement.STOP
        state = make_state(level=1)
        state.vision = make_vision("player linemate")

        conn = make_conn()
        await s._seek(state, conn, "linemate")

        conn.send.assert_not_called()
