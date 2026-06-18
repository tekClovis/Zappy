/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Parser
*/

#include "Parser.hpp"
#include <algorithm>
#include <cstdio>
#include <sstream>

static int parseId(const std::string& s)
{
    return std::stoi(s.substr(1)); // "#n" -> n
}

void Parser::parse(const std::string& line)
{
    if (line.size() < 3) return;
    auto tag = line.substr(0, 3);

    if (tag == "msz") {
        sscanf(line.c_str(), "msz %d %d", &_s.width, &_s.height);
        _s.resize(_s.width, _s.height);

    } else if (tag == "bct") {
        int x, y;
        std::array<int, 7> r{};
        sscanf(line.c_str(), "bct %d %d %d %d %d %d %d %d %d",
               &x, &y, &r[0], &r[1], &r[2], &r[3], &r[4], &r[5], &r[6]);
        if (x >= 0 && x < _s.width && y >= 0 && y < _s.height)
            _s.tile(x, y).res = r;

    } else if (tag == "tna") {
        _s.teams.push_back(line.substr(4));

    } else if (tag == "pnw") {
        Player p;
        char team[128]{};
        sscanf(line.c_str(), "pnw #%d %d %d %d %d %127s",
               &p.id, &p.x, &p.y, &p.orientation, &p.level, team);
        p.team = team;
        _s.players[p.id] = p;

    } else if (tag == "ppo") {
        int n, x, y, o;
        sscanf(line.c_str(), "ppo #%d %d %d %d", &n, &x, &y, &o);
        if (_s.players.count(n)) {
            _s.players[n].x = x;
            _s.players[n].y = y;
            _s.players[n].orientation = o;
        }

    } else if (tag == "plv") {
        int n, l;
        sscanf(line.c_str(), "plv #%d %d", &n, &l);
        if (_s.players.count(n))
            _s.players[n].level = l;

    } else if (tag == "pin") {
        int n, x, y;
        std::array<int, 7> r{};
        sscanf(line.c_str(), "pin #%d %d %d %d %d %d %d %d %d %d",
               &n, &x, &y, &r[0], &r[1], &r[2], &r[3], &r[4], &r[5], &r[6]);
        if (_s.players.count(n))
            _s.players[n].inventory = r;

    } else if (tag == "pic") {
        // start of an incantation on a tile + mark participating players
        std::istringstream ss(line.substr(4));
        int x, y, l;
        ss >> x >> y >> l;
        _s.incantations.push_back({ x, y, l });
        std::string tok;
        while (ss >> tok)
            if (!tok.empty() && tok[0] == '#' && _s.players.count(parseId(tok)))
                _s.players[parseId(tok)].incanting = true;

    } else if (tag == "pie") {
        // end of an incantation: remove it, flash the result, unmark players
        int x, y, r = 0;
        sscanf(line.c_str(), "pie %d %d %d", &x, &y, &r);
        std::erase_if(_s.incantations, [x, y](const Incantation& i) {
            return i.x == x && i.y == y;
        });
        _s.incantResults.push_back({ x, y, r != 0, 2.5f });
        for (auto& [id, p] : _s.players)
            if (p.x == x && p.y == y)
                p.incanting = false;

    } else if (tag == "pdr") {
        int n, i;
        sscanf(line.c_str(), "pdr #%d %d", &n, &i);
        if (_s.players.count(n) && i >= 0 && i < 7) {
            auto& p = _s.players[n];
            _s.tile(p.x, p.y).res[i]++;
        }

    } else if (tag == "pgt") {
        int n, i;
        sscanf(line.c_str(), "pgt #%d %d", &n, &i);
        if (_s.players.count(n) && i >= 0 && i < 7) {
            auto& p = _s.players[n];
            _s.tile(p.x, p.y).res[i] = std::max(0, _s.tile(p.x, p.y).res[i] - 1);
        }

    } else if (tag == "pdi") {
        int n;
        sscanf(line.c_str(), "pdi #%d", &n);
        _s.players.erase(n);

    } else if (tag == "enw") {
        Egg e;
        sscanf(line.c_str(), "enw #%d #%d %d %d", &e.id, &e.playerId, &e.x, &e.y);
        _s.eggs[e.id] = e;

    } else if (tag == "ebo") {
        int e;
        sscanf(line.c_str(), "ebo #%d", &e);
        _s.eggs.erase(e);

    } else if (tag == "edi") {
        int e;
        sscanf(line.c_str(), "edi #%d", &e);
        _s.eggs.erase(e);

    } else if (tag == "sgt") {
        sscanf(line.c_str(), "sgt %d", &_s.timeUnit);

    } else if (tag == "sst") {
        sscanf(line.c_str(), "sst %d", &_s.timeUnit);

    } else if (tag == "seg") {
        _s.over = true;
        if (line.size() > 4) _s.winner = line.substr(4);

    } else if (tag == "smg") {
        if (line.size() > 4) {
            _s.serverMessages.push_back(line.substr(4));
            if (_s.serverMessages.size() > 50)
                _s.serverMessages.erase(_s.serverMessages.begin());
        }
    }
    // pex, pbc, pfk, suc, sbp handled silently
}
