##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## test_game_state.py
##

import pytest
from src.state.game_state import GameState, LEVEL_3_TO_8_REQUIREMENTS, RESOURCES


def make_state(level=1, inventory=None):
    state = GameState(world_x=10, world_y=10, client_num=1)
    state.level = level
    if inventory:
        state.inventory.update(inventory)
    return state


class TestHasResourcesForElevation:
    def test_level1_exact_resources(self):
        state = make_state(level=1, inventory={"linemate": 1})
        assert state.has_resources_for_elevation()

    def test_level1_missing_linemate(self):
        state = make_state(level=1, inventory={"linemate": 0})
        assert not state.has_resources_for_elevation()

    def test_level1_excess_resources(self):
        state = make_state(level=1, inventory={"linemate": 5})
        assert state.has_resources_for_elevation()

    def test_level2_all_present(self):
        state = make_state(level=2, inventory={"linemate": 1, "deraumere": 1, "sibur": 1})
        assert state.has_resources_for_elevation()

    def test_level2_missing_deraumere(self):
        state = make_state(level=2, inventory={"linemate": 1, "deraumere": 0, "sibur": 1})
        assert not state.has_resources_for_elevation()

    def test_level3_all_present(self):
        state = make_state(level=3, inventory={"linemate": 2, "sibur": 1, "phiras": 2})
        assert state.has_resources_for_elevation()

    def test_level3_insufficient_linemate(self):
        state = make_state(level=3, inventory={"linemate": 1, "sibur": 1, "phiras": 2})
        assert not state.has_resources_for_elevation()

    def test_level4_all_present(self):
        state = make_state(level=4, inventory={"linemate": 1, "deraumere": 1, "sibur": 2, "phiras": 1})
        assert state.has_resources_for_elevation()

    def test_level5_all_present(self):
        state = make_state(level=5, inventory={"linemate": 1, "deraumere": 2, "sibur": 1, "mendiane": 3})
        assert state.has_resources_for_elevation()

    def test_level5_missing_mendiane(self):
        state = make_state(level=5, inventory={"linemate": 1, "deraumere": 2, "sibur": 1, "mendiane": 2})
        assert not state.has_resources_for_elevation()

    def test_level6_all_present(self):
        state = make_state(level=6, inventory={"linemate": 1, "deraumere": 2, "sibur": 3, "phiras": 1})
        assert state.has_resources_for_elevation()

    def test_level7_all_present(self):
        state = make_state(level=7, inventory={
            "linemate": 2, "deraumere": 2, "sibur": 2,
            "mendiane": 2, "phiras": 2, "thystame": 1
        })
        assert state.has_resources_for_elevation()

    def test_level7_missing_thystame(self):
        state = make_state(level=7, inventory={
            "linemate": 2, "deraumere": 2, "sibur": 2,
            "mendiane": 2, "phiras": 2, "thystame": 0
        })
        assert not state.has_resources_for_elevation()

    def test_unknown_level_returns_true(self):
        state = make_state(level=8)
        assert state.has_resources_for_elevation()


class TestPlayersNeededForElevation:
    @pytest.mark.parametrize("level,expected", [
        (1, 1),
        (2, 2),
        (3, 2),
        (4, 4),
        (5, 4),
        (6, 6),
        (7, 6),
    ])
    def test_all_levels(self, level, expected):
        state = make_state(level=level)
        assert state.players_needed_for_elevation() == expected

    def test_unknown_level_defaults_to_1(self):
        state = make_state(level=99)
        assert state.players_needed_for_elevation() == 1


class TestResourcesMissingForElevation:
    def test_level1_nothing_missing(self):
        state = make_state(level=1, inventory={"linemate": 1})
        assert state.resources_missing_for_elevation() == {}

    def test_level1_linemate_missing(self):
        state = make_state(level=1, inventory={"linemate": 0})
        assert state.resources_missing_for_elevation() == {"linemate": 1}

    def test_level2_partial_missing(self):
        state = make_state(level=2, inventory={"linemate": 1, "deraumere": 0, "sibur": 1})
        missing = state.resources_missing_for_elevation()
        assert missing == {"deraumere": 1}

    def test_level2_all_missing(self):
        state = make_state(level=2)
        missing = state.resources_missing_for_elevation()
        assert missing["linemate"] == 1
        assert missing["deraumere"] == 1
        assert missing["sibur"] == 1

    def test_level3_partial_missing(self):
        state = make_state(level=3, inventory={"linemate": 1, "sibur": 1, "phiras": 0})
        missing = state.resources_missing_for_elevation()
        assert missing == {"linemate": 1, "phiras": 2}

    def test_players_key_excluded(self):
        state = make_state(level=1)
        missing = state.resources_missing_for_elevation()
        assert "players" not in missing

    def test_excess_not_in_missing(self):
        state = make_state(level=1, inventory={"linemate": 5})
        assert state.resources_missing_for_elevation() == {}


class TestHasResourcesForLevel3To8:
    def _full_inventory(self):
        return dict(LEVEL_3_TO_8_REQUIREMENTS)

    def test_all_resources_present(self):
        state = make_state(inventory=self._full_inventory())
        assert state.has_resources_for_level_3_to_8()

    def test_missing_one_resource(self):
        inv = self._full_inventory()
        inv["thystame"] = 0
        state = make_state(inventory=inv)
        assert not state.has_resources_for_level_3_to_8()

    def test_all_zero(self):
        state = make_state()
        assert not state.has_resources_for_level_3_to_8()

    def test_excess_resources_still_ok(self):
        inv = {k: v + 10 for k, v in LEVEL_3_TO_8_REQUIREMENTS.items()}
        state = make_state(inventory=inv)
        assert state.has_resources_for_level_3_to_8()

    def test_missing_linemate(self):
        inv = self._full_inventory()
        inv["linemate"] = LEVEL_3_TO_8_REQUIREMENTS["linemate"] - 1
        state = make_state(inventory=inv)
        assert not state.has_resources_for_level_3_to_8()


class TestResourcesMissingForLevel3To8:
    def test_nothing_missing(self):
        inv = dict(LEVEL_3_TO_8_REQUIREMENTS)
        state = make_state(inventory=inv)
        assert state.resources_missing_for_level_3_to_8() == {}

    def test_all_missing(self):
        state = make_state()
        missing = state.resources_missing_for_level_3_to_8()
        for resource, qty in LEVEL_3_TO_8_REQUIREMENTS.items():
            assert missing[resource] == qty

    def test_partial_missing(self):
        inv = dict(LEVEL_3_TO_8_REQUIREMENTS)
        inv["sibur"] = LEVEL_3_TO_8_REQUIREMENTS["sibur"] - 3
        state = make_state(inventory=inv)
        missing = state.resources_missing_for_level_3_to_8()
        assert missing == {"sibur": 3}

    def test_excess_not_in_missing(self):
        inv = {k: v + 5 for k, v in LEVEL_3_TO_8_REQUIREMENTS.items()}
        state = make_state(inventory=inv)
        assert state.resources_missing_for_level_3_to_8() == {}


class TestGameStateInit:
    def test_initial_inventory_all_zero(self):
        state = GameState(world_x=10, world_y=10, client_num=2)
        for resource in RESOURCES:
            assert state.inventory[resource] == 0

    def test_initial_level(self):
        state = GameState(world_x=10, world_y=10, client_num=1)
        assert state.level == 1

    def test_initial_alive(self):
        state = GameState(world_x=10, world_y=10, client_num=1)
        assert state.alive is True

    def test_world_dimensions(self):
        state = GameState(world_x=20, world_y=15, client_num=3)
        assert state.world_x == 20
        assert state.world_y == 15
