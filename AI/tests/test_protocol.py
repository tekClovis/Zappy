##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## test_protocol.py
##

from src.client.protocol import (
    parse_response,
    parse_unsolicited,
    is_unsolicited,
    ServerMsg,
    LookResponse,
    InventoryResponse,
    MessageEvent,
    EjectEvent,
    CurrentLevelEvent,
)


class TestIsUnsolicited:
    def test_message(self):
        assert is_unsolicited("message 3, hello")

    def test_eject(self):
        assert is_unsolicited("eject: 2")

    def test_current_level(self):
        assert is_unsolicited("Current level: 4")

    def test_dead(self):
        assert is_unsolicited("dead")

    def test_ok_is_not_unsolicited(self):
        assert not is_unsolicited("ok")

    def test_ko_is_not_unsolicited(self):
        assert not is_unsolicited("ko")

    def test_look_is_not_unsolicited(self):
        assert not is_unsolicited("[player, linemate,]")

    def test_inventory_is_not_unsolicited(self):
        assert not is_unsolicited("[food 10, linemate 0,]")

    def test_elevation_underway_is_not_unsolicited(self):
        assert not is_unsolicited("Elevation underway")


class TestParseResponse:
    def test_ok(self):
        assert parse_response("ok") is ServerMsg.OK

    def test_ko(self):
        assert parse_response("ko") is ServerMsg.KO

    def test_dead(self):
        assert parse_response("dead") is ServerMsg.DEAD

    def test_elevation_underway(self):
        assert parse_response("Elevation underway") is ServerMsg.ELEVATION_UNDERWAY

    def test_connect_nbr(self):
        assert parse_response("5") == 5

    def test_connect_nbr_zero(self):
        assert parse_response("0") == 0

    def test_look_empty_tiles(self):
        result = parse_response("[,]")
        assert isinstance(result, LookResponse)
        assert result.tiles == ((), ())

    def test_look_player_on_tile(self):
        result = parse_response("[player,]")
        assert isinstance(result, LookResponse)
        assert "player" in result.tiles[0]

    def test_look_multiple_tiles(self):
        result = parse_response("[player, linemate, food deraumere,]")
        assert isinstance(result, LookResponse)
        assert "player" in result.tiles[0]
        assert "linemate" in result.tiles[1]
        assert "food" in result.tiles[2]
        assert "deraumere" in result.tiles[2]

    def test_look_current_tile_helper(self):
        result = parse_response("[player linemate,]")
        assert isinstance(result, LookResponse)
        assert result.current_tile() == ("player", "linemate")

    def test_look_empty_current_tile_helper(self):
        result = parse_response("[,]")
        assert isinstance(result, LookResponse)
        assert result.current_tile() == ()

    def test_inventory(self):
        result = parse_response("[food 10, linemate 2, deraumere 0, sibur 1, mendiane 0, phiras 3, thystame 0]")
        assert isinstance(result, InventoryResponse)
        assert result.resources["food"] == 10
        assert result.resources["linemate"] == 2
        assert result.resources["deraumere"] == 0
        assert result.resources["sibur"] == 1
        assert result.resources["phiras"] == 3
        assert result.resources["thystame"] == 0

    def test_inventory_single_item(self):
        result = parse_response("[food 345]")
        assert isinstance(result, InventoryResponse)
        assert result.resources["food"] == 345

    def test_ok_with_trailing_whitespace(self):
        assert parse_response("ok  ") is ServerMsg.OK

    def test_ok_with_leading_whitespace(self):
        assert parse_response("  ok") is ServerMsg.OK


class TestParseUnsolicited:
    def test_message_direction_0(self):
        event = parse_unsolicited("message 0, hello world")
        assert isinstance(event, MessageEvent)
        assert event.direction == 0
        assert event.text == "hello world"

    def test_message_direction_nonzero(self):
        event = parse_unsolicited("message 3, HELP_2_1234")
        assert isinstance(event, MessageEvent)
        assert event.direction == 3
        assert event.text == "HELP_2_1234"

    def test_message_text_with_comma(self):
        event = parse_unsolicited("message 1, hello, world")
        assert isinstance(event, MessageEvent)
        assert event.direction == 1
        assert event.text == "hello, world"

    def test_eject(self):
        event = parse_unsolicited("eject: 5")
        assert isinstance(event, EjectEvent)
        assert event.direction == 5

    def test_current_level(self):
        event = parse_unsolicited("Current level: 3")
        assert isinstance(event, CurrentLevelEvent)
        assert event.level == 3

    def test_current_level_max(self):
        event = parse_unsolicited("Current level: 8")
        assert isinstance(event, CurrentLevelEvent)
        assert event.level == 8

    def test_dead(self):
        result = parse_unsolicited("dead")
        assert result is ServerMsg.DEAD
