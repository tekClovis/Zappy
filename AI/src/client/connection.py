##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## connection.py
##

import asyncio
from typing import Any, Callable

from .protocol import (
    parse_response,
    parse_unsolicited,
    is_unsolicited,
    ServerMsg,
)


class ZappyConnection:
    """Async TCP client for the Zappy server.

    Maintains a request pipeline of up to 10 in-flight commands (server
    constraint). A background reader task dispatches server lines to the
    correct pending Future or to the unsolicited-event callback.
    """

    _MAX_PIPELINE = 10

    def __init__(self, host: str, port: int, team_name: str) -> None:
        self._host = host
        self._port = port
        self._team_name = team_name

        self._reader: asyncio.StreamReader | None = None
        self._writer: asyncio.StreamWriter | None = None

        # Guards pipeline depth: at most _MAX_PIPELINE futures in flight.
        self._pipeline = asyncio.Semaphore(self._MAX_PIPELINE)
        # FIFO queue of Futures waiting for their response.
        self._pending: asyncio.Queue[asyncio.Future[Any]] = asyncio.Queue()

        self._unsolicited_cb: Callable[[Any], None] | None = None
        self._reader_task: asyncio.Task | None = None


    def on_unsolicited(self, callback: Callable[[Any], None]) -> None:
        """Register a callback for spontaneous server events (message, eject, dead…)."""
        self._unsolicited_cb = callback

    async def connect(self) -> tuple[int, int, int]:
        """Open the TCP socket and perform the handshake.

        Returns (client_num, world_x, world_y).
        """
        self._reader, self._writer = await asyncio.open_connection(
            self._host, self._port
        )

        async def _tread() -> str:
            return await asyncio.wait_for(self._readline(), timeout=10.0)

        welcome = await _tread()
        if welcome != "WELCOME":
            raise ConnectionError(f"Expected WELCOME, got {welcome!r}")

        await self._writeline(self._team_name)
        client_num_str = await _tread()
        if client_num_str == "ko":
            # Server has no available slot for this team right now.
            self._writer.close()
            raise ConnectionError("no_slot")
        client_num = int(client_num_str)
        x, y = map(int, (await _tread()).split())

        self._reader_task = asyncio.create_task(
            self._reader_loop(), name="zappy-reader"
        )
        return client_num, x, y

    async def send(self, command: str) -> Any:
        """Send a command and await its response.

        Blocks if the 10-command pipeline is already full.
        Raises asyncio.CancelledError if the connection closes before the
        response arrives (e.g. player died).
        """
        await self._pipeline.acquire()

        loop = asyncio.get_running_loop()
        fut: asyncio.Future[Any] = loop.create_future()
        await self._pending.put(fut)

        await self._writeline(command)
        return await fut

    async def close(self) -> None:
        if self._reader_task:
            self._reader_task.cancel()
            try:
                await asyncio.wait_for(self._reader_task, timeout=1.0)
            except (asyncio.CancelledError, asyncio.TimeoutError):
                pass
        if self._writer:
            try:
                self._writer.close()
                await asyncio.wait_for(self._writer.wait_closed(), timeout=2.0)
            except (asyncio.TimeoutError, Exception):
                pass


    async def _reader_loop(self) -> None:
        try:
            while True:
                line = await self._readline()

                if not line:
                    break

                if is_unsolicited(line):
                    event = parse_unsolicited(line)
                    if self._unsolicited_cb:
                        self._unsolicited_cb(event)
                    if event is ServerMsg.DEAD:
                        break
                else:
                    parsed = parse_response(line)
                    fut: asyncio.Future[Any] = await self._pending.get()
                    if not fut.done():
                        fut.set_result(parsed)
                    self._pipeline.release()
        except (asyncio.IncompleteReadError, ConnectionResetError, OSError):
            pass
        finally:
            self._drain_pending()

    def _drain_pending(self) -> None:
        """Cancel all futures still waiting for a response after disconnect."""
        while not self._pending.empty():
            try:
                fut = self._pending.get_nowait()
                if not fut.done():
                    fut.cancel()
                self._pipeline.release()
            except asyncio.QueueEmpty:
                break

    async def _readline(self) -> str:
        assert self._reader is not None
        line = await self._reader.readline()
        return line.decode().rstrip("\n\r")

    async def _writeline(self, msg: str) -> None:
        assert self._writer is not None
        self._writer.write(f"{msg}\n".encode())
        await self._writer.drain()
