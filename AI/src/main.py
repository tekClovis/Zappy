##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## main.py
##

from __future__ import annotations
import argparse
import asyncio
import sys

from .client.connection import ZappyConnection
from .client.protocol import CurrentLevelEvent, ServerMsg
from .state.game_state import GameState
from .strategy.heuristic import HeuristicStrategy


def _parse_args():
    p = argparse.ArgumentParser(prog="zappy_ai", add_help=False)
    p.add_argument("-p", type=int, required=True, dest="port", help="server port")
    p.add_argument("-n", required=True, dest="name", help="team name")
    p.add_argument("-h", default="localhost", dest="host", help="server hostname")
    p.add_argument("--help", action="help", help="show this help message and exit")
    return p.parse_args()


async def _run(host: str, port: int, team: str) -> None:
    conn = ZappyConnection(host, port, team)
    state = GameState(world_x=0, world_y=0, client_num=0)

    strategy = HeuristicStrategy()

    def _on_unsolicited(event) -> None:
        if isinstance(event, CurrentLevelEvent):
            state.level = event.level
        elif event is ServerMsg.DEAD:
            state.alive = False
        strategy.handle_event(event)

    conn.on_unsolicited(_on_unsolicited)

    client_num, x, y = await conn.connect()

    if x is None and y is None:
        await conn.close()
        raise ConnectionError(f"Team {team} is full")

    state.world_x = x
    state.world_y = y
    state.client_num = client_num

    await state.get_freq(conn)

    try:
        while state.alive:
            await strategy.tick(state, conn)
    except asyncio.CancelledError:
        pass
    finally:
        await conn.close()


def main() -> None:
    args = _parse_args()
    try:
        asyncio.run(_run(args.host, args.port, args.name))
    except KeyboardInterrupt:
        sys.exit(0)
    except ConnectionError as e:
        print(f"ConnectionError: {e}")
        sys.exit(84)
