##
## EPITECH PROJECT, 2026
## G-YEP-400-LIL-4-1-zappy-11
## File description:
## test_heuristic_state.py — tests for handle_event() state transitions
##

from src.client.protocol import CurrentLevelEvent, MessageEvent
from src.strategy.heuristic import HeuristicStrategy, Role, Movement


def make_strategy(pid="1234", level=1):
    s = HeuristicStrategy()
    s._pid = pid
    s._level = level
    return s


def msg(direction, text):
    return MessageEvent(direction, text)


class TestCurrentLevelEvent:
    def test_updates_level(self):
        s = make_strategy()
        s.handle_event(CurrentLevelEvent(3))
        assert s._level == 3

    def test_sets_elevation_done(self):
        s = make_strategy()
        assert not s._elevation_done.is_set()
        s.handle_event(CurrentLevelEvent(2))
        assert s._elevation_done.is_set()

    def test_respond_to_br_set(self):
        s = make_strategy()
        s._respond_to_br = False
        s.handle_event(CurrentLevelEvent(2))
        assert s._respond_to_br

    def test_leader_direction_reset_below_4(self):
        s = make_strategy(level=3)
        s._leader_direction = 5
        s.handle_event(CurrentLevelEvent(3))
        assert s._leader_direction is None

    def test_leader_direction_set_to_0_at_4_plus(self):
        s = make_strategy(level=3)
        s._leader_direction = None
        s.handle_event(CurrentLevelEvent(4))
        assert s._leader_direction == 0


class TestPlayerBroadcast:
    def test_registers_new_player(self):
        s = make_strategy(pid="9999")
        s.handle_event(msg(1, "PLAYER_2_1234"))
        assert s._others.get("1234") == 2

    def test_updates_existing_player(self):
        s = make_strategy(pid="9999")
        s._others["1234"] = 1
        s.handle_event(msg(0, "PLAYER_3_1234"))
        assert s._others["1234"] == 3

    def test_sets_respond_to_br_for_new_player(self):
        s = make_strategy(pid="9999")
        s._respond_to_br = False
        s.handle_event(msg(1, "PLAYER_1_5678"))
        assert s._respond_to_br

    def test_ignores_invalid_level(self):
        s = make_strategy(pid="9999")
        s.handle_event(msg(1, "PLAYER_X_1234"))
        assert "1234" not in s._others

    def test_ignores_invalid_pid(self):
        s = make_strategy(pid="9999")
        s.handle_event(msg(1, "PLAYER_2_notapid"))
        assert "notapid" not in s._others

    def test_accepts_leader_pid(self):
        s = make_strategy(pid="9999")
        s.handle_event(msg(0, "PLAYER_3_LEADER"))
        assert s._others.get("LEADER") == 3


class TestHelpBroadcast:
    def test_sets_pid_to_help(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(3, "HELP_2_1234"))
        assert s._pid_to_help == "1234"

    def test_sets_mate_direction(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(5, "HELP_2_1234"))
        assert s._mate_direction == 5

    def test_stops_movement_when_direction_0(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(0, "HELP_2_1234"))
        assert s._movement == Movement.STOP

    def test_keeps_running_when_direction_nonzero(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(2, "HELP_2_1234"))
        assert s._movement == Movement.RUN

    def test_ignores_different_level(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(1, "HELP_3_5678"))
        assert s._pid_to_help == ""

    def test_ignores_second_helper_once_one_set(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(1, "HELP_2_1111"))
        s.handle_event(msg(2, "HELP_2_2222"))
        assert s._pid_to_help == "1111"

    def test_clears_wait_broadcast(self):
        s = make_strategy(pid="9999", level=2)
        s._wait_broadcast = True
        s.handle_event(msg(3, "HELP_2_1234"))
        assert not s._wait_broadcast


class TestHelp2Broadcast:
    def test_sets_pid_to_help_when_addressed(self):
        s = make_strategy(pid="1234", level=2)
        s.handle_event(msg(4, "HELP2_1234_2_5678"))
        assert s._pid_to_help == "5678"

    def test_ignored_when_not_addressed(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(4, "HELP2_1234_2_5678"))
        assert s._pid_to_help == ""

    def test_direction_set(self):
        s = make_strategy(pid="1234", level=2)
        s.handle_event(msg(6, "HELP2_1234_2_5678"))
        assert s._mate_direction == 6


class TestHereForHelpBroadcast:
    def test_sets_helper_pid_when_addressed(self):
        s = make_strategy(pid="1234", level=2)
        s.handle_event(msg(0, "HERE_FOR_HELP_1234_2_5678"))
        assert s._helper_pid == "5678"

    def test_ignored_when_not_addressed(self):
        s = make_strategy(pid="9999", level=2)
        s.handle_event(msg(0, "HERE_FOR_HELP_1234_2_5678"))
        assert s._helper_pid == ""

    def test_ignored_when_level_mismatch(self):
        s = make_strategy(pid="1234", level=2)
        s.handle_event(msg(0, "HERE_FOR_HELP_1234_3_5678"))
        assert s._helper_pid == ""

    def test_first_helper_wins(self):
        s = make_strategy(pid="1234", level=2)
        s.handle_event(msg(0, "HERE_FOR_HELP_1234_2_5678"))
        s.handle_event(msg(0, "HERE_FOR_HELP_1234_2_9012"))
        assert s._helper_pid == "5678"


class TestSuccessHelpBroadcast:
    def test_clears_pid_to_help(self):
        s = make_strategy(pid="9999", level=2)
        s._pid_to_help = "1234"
        s.handle_event(msg(0, "SUCCESS_HELP_2_1234"))
        assert s._pid_to_help == ""

    def test_clears_mate_direction(self):
        s = make_strategy(pid="9999", level=2)
        s._pid_to_help = "1234"
        s._mate_direction = 3
        s.handle_event(msg(0, "SUCCESS_HELP_2_1234"))
        assert s._mate_direction is None

    def test_resumes_running(self):
        s = make_strategy(pid="9999", level=2)
        s._pid_to_help = "1234"
        s._movement = Movement.STOP
        s.handle_event(msg(0, "SUCCESS_HELP_2_1234"))
        assert s._movement == Movement.RUN

    def test_ignored_when_not_the_helped(self):
        s = make_strategy(pid="9999", level=2)
        s._pid_to_help = "1234"
        s.handle_event(msg(0, "SUCCESS_HELP_2_5678"))
        assert s._pid_to_help == "1234"


class TestIAmLeaderBroadcast:
    def test_registers_leader(self):
        s = make_strategy(pid="9999", level=3)
        s.handle_event(msg(2, "I_AM_LEADER_3_1234"))
        assert s._others.get("LEADER") == 3
        assert s._pid_leader == "LEADER"

    def test_becomes_follower_at_level_3(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.SOLO
        s.handle_event(msg(2, "I_AM_LEADER_3_1234"))
        assert s._role == Role.FOLLOWER

    def test_no_follower_below_level_3(self):
        s = make_strategy(pid="9999", level=2)
        s._role = Role.SOLO
        s.handle_event(msg(2, "I_AM_LEADER_3_1234"))
        assert s._role == Role.SOLO

    def test_ignored_when_already_leader(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.LEADER
        s.handle_event(msg(2, "I_AM_LEADER_3_1234"))
        assert s._role == Role.LEADER

    def test_old_pid_removed_from_others(self):
        s = make_strategy(pid="9999", level=3)
        s._others["1234"] = 3
        s.handle_event(msg(2, "I_AM_LEADER_3_1234"))
        assert "1234" not in s._others


class TestIncantBroadcast:
    def test_sets_leader_direction(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.FOLLOWER
        s._pid_leader = "LEADER"
        s._others["LEADER"] = 3
        s.handle_event(msg(4, "INCANT_3_LEADER"))
        assert s._leader_direction == 4

    def test_stops_movement_when_direction_0(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.FOLLOWER
        s._pid_leader = "LEADER"
        s._others["LEADER"] = 3
        s.handle_event(msg(0, "INCANT_3_LEADER"))
        assert s._movement == Movement.STOP

    def test_ignored_when_level_below_3(self):
        s = make_strategy(pid="9999", level=2)
        s._role = Role.SOLO
        s.handle_event(msg(1, "INCANT_3_LEADER"))
        assert s._leader_direction is None

    def test_ignored_when_not_from_leader(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.FOLLOWER
        s._pid_leader = "LEADER"
        s.handle_event(msg(1, "INCANT_3_5678"))
        assert s._leader_direction is None

    def test_ignored_when_level_mismatch(self):
        s = make_strategy(pid="9999", level=3)
        s._role = Role.FOLLOWER
        s._pid_leader = "LEADER"
        s.handle_event(msg(1, "INCANT_4_LEADER"))
        assert s._leader_direction is None
