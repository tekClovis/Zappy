##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## base.py
##

from __future__ import annotations
from abc import ABC, abstractmethod

from ..client.connection import ZappyConnection
from ..state.game_state import GameState


class Strategy(ABC):
    """Decision-making interface.

    Each call to tick() receives the current game state and the connection,
    sends zero or more commands, and returns.  The event loop calls it again
    as soon as it completes.

    handle_event() is called for every unsolicited server notification
    (broadcast, ejection, level-up, death).  Override it when the strategy
    needs to react to these events — e.g. to signal an asyncio.Event that
    tick() is waiting on.
    """

    @abstractmethod
    async def tick(self, state: GameState, conn: ZappyConnection) -> None:
        ...

    def handle_event(self, event: object) -> None:
        """Called from the unsolicited-event callback.  Default: no-op."""
