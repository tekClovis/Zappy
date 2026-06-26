/*
** EPITECH PROJECT, 2026
** G-YEP-400-LIL-4-1-zappy-11
** File description:
** test_parser.cpp — unit tests for CommandDispatcher and GameState
*/

#include <cassert>
#include <iostream>
#include "../game/GameState.hpp"
#include "../protocol/Dispatcher.hpp"

static int _pass = 0;
static int _fail = 0;

#define CHECK(cond) \
    do { \
        if (cond) { _pass++; } \
        else { _fail++; \
            std::cerr << "FAIL: " #cond " (line " << __LINE__ << ")\n"; } \
    } while (0)


static void test_resize_sets_dimensions()
{
    GameState s;
    s.resize(10, 5);
    CHECK(s.width() == 10);
    CHECK(s.height() == 5);
}

static void test_resize_tiles_zeroed()
{
    GameState s;
    s.resize(3, 3);
    for (int i = 0; i < 7; i++)
        CHECK(s.tile(0, 0).res[i] == 0);
}

static void test_resize_idempotent()
{
    GameState s;
    s.resize(5, 5);
    s.setTileResources(2, 2, {1,2,3,4,5,6,7});
    s.resize(5, 5); // same size: map must NOT be wiped
    CHECK(s.tile(2, 2).res[0] == 1);
}


static void test_msz_sets_size()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 20");
    CHECK(s.width() == 10);
    CHECK(s.height() == 20);
}


static void test_bct_sets_tile_resources()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 5 5");
    d.dispatch("bct 2 3 1 2 3 4 5 6 7");
    CHECK(s.tile(2, 3).res[0] == 1);
    CHECK(s.tile(2, 3).res[6] == 7);
}

static void test_bct_out_of_bounds_ignored()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 5 5");
    d.dispatch("bct 10 10 1 2 3 4 5 6 7");
    CHECK(s.tile(0, 0).res[0] == 0);
}


static void test_tna_registers_teams()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("tna TeamA");
    d.dispatch("tna TeamB");
    CHECK(s.teams().size() == 2);
    CHECK(s.teams()[0] == "TeamA");
    CHECK(s.teams()[1] == "TeamB");
}

static void test_tna_deduplicates()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("tna TeamA");
    d.dispatch("tna TeamA");
    CHECK(s.teams().size() == 1);
}


static void test_pnw_registers_player()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 3 4 2 1 TeamA");
    CHECK(s.players().count(1));
    CHECK(s.players().at(1).x == 3);
    CHECK(s.players().at(1).y == 4);
    CHECK(s.players().at(1).orientation == 2);
    CHECK(s.players().at(1).level == 1);
    CHECK(s.players().at(1).team == "TeamA");
}


static void test_ppo_updates_position()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    d.dispatch("ppo #1 5 6 3");
    CHECK(s.players().at(1).x == 5);
    CHECK(s.players().at(1).y == 6);
    CHECK(s.players().at(1).orientation == 3);
}

static void test_ppo_unknown_player_ignored()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("ppo #99 5 6 3");
    CHECK(s.players().count(99) == 0);
}


static void test_plv_updates_level()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #2 0 0 1 1 TeamA");
    d.dispatch("plv #2 5");
    CHECK(s.players().at(2).level == 5);
}


static void test_pin_updates_inventory()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    d.dispatch("pin #1 0 0 10 2 0 0 0 0 1");
    CHECK(s.players().at(1).inventory[0] == 10);
    CHECK(s.players().at(1).inventory[6] == 1);
}


static void test_pic_marks_players_incanting()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 2 3 1 2 TeamA");
    d.dispatch("pnw #2 2 3 1 2 TeamA");
    d.dispatch("pic 2 3 2 #1 #2");
    CHECK(s.players().at(1).incanting == true);
    CHECK(s.players().at(2).incanting == true);
}

static void test_pie_clears_incanting_on_tile()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 2 3 1 2 TeamA");
    d.dispatch("pic 2 3 2 #1");
    CHECK(s.players().at(1).incanting == true);
    d.dispatch("pie 2 3 1");
    CHECK(s.players().at(1).incanting == false);
}

static void test_pie_adds_incant_result()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 2 3 1 2 TeamA");
    d.dispatch("pic 2 3 2 #1");
    d.dispatch("pie 2 3 1");
    CHECK(!s.incantResults().empty());
    CHECK(s.incantResults()[0].x == 2);
    CHECK(s.incantResults()[0].y == 3);
    CHECK(s.incantResults()[0].success == true);
}


static void test_pdr_adds_resource_to_tile()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 3 3 1 1 TeamA");
    d.dispatch("pdr #1 1");
    CHECK(s.tile(3, 3).res[1] == 1);
}

static void test_pgt_removes_resource_from_tile()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("bct 3 3 0 2 0 0 0 0 0");
    d.dispatch("pnw #1 3 3 1 1 TeamA");
    d.dispatch("pgt #1 1");
    CHECK(s.tile(3, 3).res[1] == 1);
}

static void test_pgt_does_not_go_below_zero()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    d.dispatch("pgt #1 0");
    CHECK(s.tile(0, 0).res[0] == 0);
}


static void test_pdi_removes_player()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    CHECK(s.players().count(1));
    d.dispatch("pdi #1");
    CHECK(s.players().count(1) == 0);
}


static void test_enw_adds_egg()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("enw #5 #1 3 4");
    CHECK(s.eggs().count(5));
    CHECK(s.eggs().at(5).x == 3);
    CHECK(s.eggs().at(5).y == 4);
    CHECK(s.eggs().at(5).playerId == 1);
}

static void test_ebo_removes_egg_on_hatch()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("enw #5 #1 3 4");
    d.dispatch("ebo #5");
    CHECK(s.eggs().count(5) == 0);
}

static void test_edi_removes_egg_on_death()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("enw #5 #1 3 4");
    d.dispatch("edi #5");
    CHECK(s.eggs().count(5) == 0);
}


static void test_sgt_sets_time_unit()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("sgt 200");
    CHECK(s.timeUnit() == 200);
}

static void test_sst_updates_time_unit()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("sst 50");
    CHECK(s.timeUnit() == 50);
}


static void test_seg_sets_game_over()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("seg TeamA");
    CHECK(s.isOver() == true);
    CHECK(s.winner() == "TeamA");
}


static void test_pex_adds_eject_effect()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    d.dispatch("pex #1");
    CHECK(!s.ejects().empty());
    CHECK(s.ejects()[0].playerId == 1);
}


static void test_pbc_adds_broadcast()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("msz 10 10");
    d.dispatch("pnw #1 0 0 1 1 TeamA");
    d.dispatch("pbc #1 hello world");
    CHECK(!s.broadcasts().empty());
    CHECK(s.broadcasts()[0].playerId == 1);
    CHECK(s.broadcasts()[0].text == "hello world");
}


static void test_smg_pushes_server_message()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("smg server is starting");
    CHECK(!s.serverMessages().empty());
    CHECK(s.serverMessages()[0] == "server is starting");
}


static void test_suc_pushes_error_message()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("suc");
    CHECK(!s.serverMessages().empty());
    CHECK(s.serverMessages()[0].find("suc") != std::string::npos);
}

static void test_sbp_pushes_error_message()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("sbp");
    CHECK(!s.serverMessages().empty());
    CHECK(s.serverMessages()[0].find("sbp") != std::string::npos);
}


static void test_empty_line_ignored()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("");
    CHECK(s.width() == 0);
}

static void test_short_line_ignored()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("ab");
    CHECK(s.width() == 0);
}

static void test_unknown_tag_ignored()
{
    GameState s;
    CommandDispatcher d(s);
    d.dispatch("xyz 1 2 3");
    CHECK(s.width() == 0);
}


int main()
{
    test_resize_sets_dimensions();
    test_resize_tiles_zeroed();
    test_resize_idempotent();
    test_msz_sets_size();
    test_bct_sets_tile_resources();
    test_bct_out_of_bounds_ignored();
    test_tna_registers_teams();
    test_tna_deduplicates();
    test_pnw_registers_player();
    test_ppo_updates_position();
    test_ppo_unknown_player_ignored();
    test_plv_updates_level();
    test_pin_updates_inventory();
    test_pic_marks_players_incanting();
    test_pie_clears_incanting_on_tile();
    test_pie_adds_incant_result();
    test_pdr_adds_resource_to_tile();
    test_pgt_removes_resource_from_tile();
    test_pgt_does_not_go_below_zero();
    test_pdi_removes_player();
    test_enw_adds_egg();
    test_ebo_removes_egg_on_hatch();
    test_edi_removes_egg_on_death();
    test_sgt_sets_time_unit();
    test_sst_updates_time_unit();
    test_seg_sets_game_over();
    test_pex_adds_eject_effect();
    test_pbc_adds_broadcast();
    test_smg_pushes_server_message();
    test_suc_pushes_error_message();
    test_sbp_pushes_error_message();
    test_empty_line_ignored();
    test_short_line_ignored();
    test_unknown_tag_ignored();

    std::cout << _pass << " passed, " << _fail << " failed\n";
    return _fail > 0 ? 1 : 0;
}
