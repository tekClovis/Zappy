###
### EPITECH PROJECT, 2026
### G-YEP-400-LIL-4-1-zappy-11
### File description:
### main.py
###
#
#from __future__ import annotations
#import argparse
#import asyncio
#import sys
#from functools import partial
#from typing import Callable
#
#from .client.connection import ZappyConnection
#from .client.protocol import CurrentLevelEvent, ServerMsg
#from .state.game_state import GameState
#from .strategy.base import Strategy
#from .strategy.heuristic import HeuristicStrategy
#
#_SIMPLE_STRATEGIES = {
#    "heuristic": HeuristicStrategy,
#}
#_ML_STRATEGIES = ("ml_gatherer", "ml_coordinator")
#
#
#def _parse_args():
#    p = argparse.ArgumentParser(prog="zappy_ai", add_help=False)
#    p.add_argument("-p", type=int, required=True, dest="port", help="server port")
#    p.add_argument("-n", required=True, dest="name", help="team name")
#    p.add_argument("-h", default="localhost", dest="host", help="server hostname")
#    p.add_argument("--strategy", default="heuristic", choices=[*_SIMPLE_STRATEGIES, *_ML_STRATEGIES], help="AI strategy to use")
#    p.add_argument("--model", default=None, dest="model", help="path to a trained .pt model (required for ml_* strategies)")
#    p.add_argument("--help", action="help", help="show this help message and exit")
#    return p.parse_args()
#
#
#def _build_factory(args: argparse.Namespace):
#    if args.strategy in _SIMPLE_STRATEGIES:
#        return _SIMPLE_STRATEGIES[args.strategy]
#
#    if args.model is None:
#        sys.exit(f"[zappy_ai] --model est requis pour la stratégie '{args.strategy}'")
#
#    from .strategy.ml_ppo import MLPPOStrategy
#    return partial(MLPPOStrategy, model_path=args.model)
#
#
#async def _run(host: str, port: int, team: str, strategy_factory) -> None:
#    conn = ZappyConnection(host, port, team)
#    state = GameState(world_x=0, world_y=0, client_num=0)
#
#    strategy = strategy_factory()
#
#    def _on_unsolicited(event) -> None:
#        if isinstance(event, CurrentLevelEvent):
#            state.level = event.level
#        elif event is ServerMsg.DEAD:
#            state.alive = False
#        strategy.handle_event(event)
#
#    conn.on_unsolicited(_on_unsolicited)
#
#    client_num, x, y = await conn.connect()
#    state.world_x = x
#    state.world_y = y
#    state.client_num = client_num
#
#    await state.get_freq(conn)
#
#    try:
#        while state.alive:
#            await strategy.tick(state, conn)
#    except asyncio.CancelledError:
#        pass
#    finally:
#        await conn.close()
#
#
#def main() -> None:
#    args = _parse_args()
#    factory = _build_factory(args)
#    try:
#        asyncio.run(_run(args.host, args.port, args.name, factory))
#    except KeyboardInterrupt:
#        sys.exit(0)
#
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
