##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## connection.py
##

from __future__ import annotations
import asyncio
from .protocol import parse_response, parse_unsolicited, is_unsolicited, ServerMsg


class ZappyConnection:
    _MAX_PIPELINE = 10

    def __init__(self, host: str, port: int, team_name: str) -> None:
        self._host = host
        self._port = port
        self._team_name = team_name

        self._reader = None
        self._writer = None

        self._current_future = None
        self._current_command = None

        self._unsolicited_cb = None
        self._reader_task = None

    def on_unsolicited(self, callback) -> None:
        self._unsolicited_cb = callback

    async def connect(self):
        self._reader, self._writer = await asyncio.open_connection(self._host, self._port)

        async def _tread() -> str:
            return await asyncio.wait_for(self._readline(), timeout=10.0)

        welcome = await _tread()
        if welcome != "WELCOME":
            raise ConnectionError(f"Expected WELCOME, got {welcome!r}")

        await self._writeline(self._team_name)
        client_num_str = await _tread()
        if client_num_str == "ko":
            self._writer.close()
            raise ConnectionError("no_slot")
        client_num = int(client_num_str)
        map_dim = await _tread()
        if not map_dim:
            return client_num, None, None
        x, y = map(int, map_dim.split())

        self._reader_task = asyncio.create_task(self._reader_loop(), name="zappy-reader")
        return client_num, x, y

    async def send(self, command):
        assert self._current_future is None, "already waiting"

        self._current_command = command
        self._current_future = asyncio.get_running_loop().create_future()

        await self._writeline(command)
        return await self._current_future

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
                    if self._current_future:
                        self._current_future.set_result(parsed)
                        self._current_future = None

        except (asyncio.IncompleteReadError, ConnectionResetError, OSError):
            pass
        finally:
            self._drain_pending()

    def _drain_pending(self) -> None:
        if self._current_future is not None and not self._current_future.done():
            self._current_future.cancel()

    async def _readline(self) -> str:
        assert self._reader is not None
        line = await self._reader.readline()
        return line.decode().rstrip("\n\r")

    async def _writeline(self, msg: str) -> None:
        assert self._writer is not None
        self._writer.write(f"{msg}\n".encode())
        await self._writer.drain()
