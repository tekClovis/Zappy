##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## test_heuristic_helpers.py
##

from src.strategy.heuristic import _tile_row_col, _tile_col_offset, _best_tile_in_vision


class TestTileRowCol:
    def test_tile_0_is_origin(self):
        assert _tile_row_col(0) == (0, 0)

    def test_tile_1_front_left(self):
        row, col = _tile_row_col(1)
        assert row == 1
        assert col == -1

    def test_tile_2_front_center(self):
        row, col = _tile_row_col(2)
        assert row == 1
        assert col == 0

    def test_tile_3_front_right(self):
        row, col = _tile_row_col(3)
        assert row == 1
        assert col == 1

    def test_tile_4_row2_leftmost(self):
        row, col = _tile_row_col(4)
        assert row == 2
        assert col == -2

    def test_tile_6_row2_center(self):
        row, col = _tile_row_col(6)
        assert row == 2
        assert col == 0

    def test_tile_8_row2_rightmost(self):
        row, col = _tile_row_col(8)
        assert row == 2
        assert col == 2

    def test_row2_has_5_tiles(self):
        cols = [_tile_row_col(i)[1] for i in range(4, 9)]
        assert cols == [-2, -1, 0, 1, 2]

    def test_tile_9_starts_row3(self):
        row, col = _tile_row_col(9)
        assert row == 3


class TestTileColOffset:
    def test_tile_0(self):
        assert _tile_col_offset(0) == 0

    def test_tile_2_center(self):
        assert _tile_col_offset(2) == 0

    def test_tile_1_negative(self):
        assert _tile_col_offset(1) < 0

    def test_tile_3_positive(self):
        assert _tile_col_offset(3) > 0

    def test_tile_6_center(self):
        assert _tile_col_offset(6) == 0

    def test_tile_4_most_left(self):
        assert _tile_col_offset(4) < _tile_col_offset(5)

    def test_tile_8_most_right(self):
        assert _tile_col_offset(8) > _tile_col_offset(7)


class TestBestTileInVision:
    def _make_vision(self, *tile_contents):
        return tuple(tuple(item for item in t.split() if item) for t in tile_contents)

    def test_returns_none_when_resource_absent(self):
        vision = self._make_vision("player", "", "food")
        assert _best_tile_in_vision(vision, "linemate") is None

    def test_returns_none_on_empty_vision(self):
        assert _best_tile_in_vision((), "linemate") is None

    def test_finds_resource_on_current_tile(self):
        vision = self._make_vision("player linemate", "")
        idx = _best_tile_in_vision(vision, "linemate")
        assert idx == 0

    def test_prefers_closer_tile(self):
        # linemate at tile 2 (row 1) vs tile 6 (row 2)
        vision = self._make_vision("player", "", "linemate", "", "", "", "linemate", "", "")
        idx = _best_tile_in_vision(vision, "linemate")
        assert idx == 2

    def test_prefers_denser_tile_when_close(self):
        # Two linemates at tile 2 vs one at tile 2 — always pick 2
        vision = self._make_vision("player", "", "linemate linemate", "linemate", "", "", "", "", "")
        idx = _best_tile_in_vision(vision, "linemate")
        assert idx == 2

    def test_prefer_density_flag_affects_score(self):
        vision = self._make_vision("player", "food food food", "food", "", "", "", "", "", "")
        idx_density = _best_tile_in_vision(vision, "food", prefer_density=True)
        idx_normal = _best_tile_in_vision(vision, "food", prefer_density=False)
        # Both should pick the same winner here (tile 1 is closest and densest)
        assert idx_density == 1
        assert idx_normal == 1

    def test_prefers_center_column_over_side(self):
        # resource on tile 1 (col -1) and tile 3 (col +1) at same row — neither is perfect
        # but tile 2 (col 0) with resource should win
        vision = self._make_vision("player", "linemate", "linemate", "linemate", "", "", "", "", "")
        idx = _best_tile_in_vision(vision, "linemate")
        assert idx == 2  # center of row 1 preferred
