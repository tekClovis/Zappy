/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Commands
*/

#include "Commands.hpp"
#include "ProtocolUtil.hpp"
#include <algorithm>

static void pushServerMessage(GameState& s, const std::string& msg)
{
    s.serverMessages.push_back(msg);
    if (s.serverMessages.size() > 50)
        s.serverMessages.erase(s.serverMessages.begin());
}

// --- map / world ---

void MapSizeCommand::execute(GameState& s, std::istringstream& a)
{
    int w = 0;
    int h = 0;
    a >> w >> h;
    s.resize(w, h);
}

void TileContentCommand::execute(GameState& s, std::istringstream& a)
{
    int x = 0;
    int y = 0;
    std::array<int, 7> r{};
    a >> x >> y;
    for (auto& v : r)
        a >> v;
    if (x >= 0 && x < s.width && y >= 0 && y < s.height)
        s.tile(x, y).res = r;
}

// --- teams / server ---

void TeamNameCommand::execute(GameState& s, std::istringstream& a)
{
    std::string name;
    a >> name;
    if (!name.empty())
        s.teams.push_back(name);
}

void TimeUnitGetCommand::execute(GameState& s, std::istringstream& a)
{
    a >> s.timeUnit;
}

void TimeUnitSetCommand::execute(GameState& s, std::istringstream& a)
{
    a >> s.timeUnit;
}

void EndGameCommand::execute(GameState& s, std::istringstream& a)
{
    s.over = true;
    s.winner = readRest(a);
}

void ServerMessageCommand::execute(GameState& s, std::istringstream& a)
{
    std::string msg = readRest(a);
    if (!msg.empty())
        pushServerMessage(s, msg);
}

void UnknownCommand::execute(GameState& s, std::istringstream&)
{
    pushServerMessage(s, "[server] unknown command (suc)");
}

void BadParameterCommand::execute(GameState& s, std::istringstream&)
{
    pushServerMessage(s, "[server] bad parameter (sbp)");
}

// --- players ---

void NewPlayerCommand::execute(GameState& s, std::istringstream& a)
{
    Player p;
    p.id = readId(a);
    a >> p.x >> p.y >> p.orientation >> p.level >> p.team;
    if (p.id >= 0)
        s.players[p.id] = p;
}

void PlayerPositionCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    auto it = s.players.find(id);
    if (it == s.players.end())
        return;
    a >> it->second.x >> it->second.y >> it->second.orientation;
}

void PlayerLevelCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    auto it = s.players.find(id);
    if (it == s.players.end())
        return;
    a >> it->second.level;
}

void PlayerInventoryCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int x = 0;
    int y = 0;
    std::array<int, 7> r{};
    a >> x >> y;
    for (auto& v : r)
        a >> v;
    auto it = s.players.find(id);
    if (it != s.players.end())
        it->second.inventory = r;
}

void PlayerDeathCommand::execute(GameState& s, std::istringstream& a)
{
    s.players.erase(readId(a));
}

void IncantationStartCommand::execute(GameState& s, std::istringstream& a)
{
    int x = 0;
    int y = 0;
    int level = 0;
    a >> x >> y >> level;
    s.incantations.push_back({ x, y, level });
    int id = -1;
    while ((id = readId(a)) >= 0) {
        auto it = s.players.find(id);
        if (it != s.players.end())
            it->second.incanting = true;
    }
}

void IncantationEndCommand::execute(GameState& s, std::istringstream& a)
{
    int x = 0;
    int y = 0;
    int result = 0;
    a >> x >> y >> result;
    std::erase_if(s.incantations, [x, y](const Incantation& i) {
        return i.x == x && i.y == y;
    });
    s.incantResults.push_back({ x, y, result != 0, 2.5f });
    for (auto& [id, p] : s.players)
        if (p.x == x && p.y == y)
            p.incanting = false;
}

void PlayerDropCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int i = -1;
    a >> i;
    auto it = s.players.find(id);
    if (it != s.players.end() && i >= 0 && i < 7)
        s.tile(it->second.x, it->second.y).res[i]++;
}

void PlayerTakeCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int i = -1;
    a >> i;
    auto it = s.players.find(id);
    if (it != s.players.end() && i >= 0 && i < 7) {
        int& q = s.tile(it->second.x, it->second.y).res[i];
        q = std::max(0, q - 1);
    }
}

void BroadcastCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    s.broadcasts.push_back({ id, readRest(a), 3.0f });
}

void EjectCommand::execute(GameState& s, std::istringstream& a)
{
    s.ejects.push_back({ readId(a), 1.0f });
}

// --- eggs ---

void EggLaidCommand::execute(GameState& s, std::istringstream& a)
{
    Egg e;
    e.id = readId(a);
    e.playerId = readId(a);
    a >> e.x >> e.y;
    if (e.id >= 0)
        s.eggs[e.id] = e;
}

void EggHatchCommand::execute(GameState& s, std::istringstream& a)
{
    auto it = s.eggs.find(readId(a));
    if (it != s.eggs.end()) {
        s.eggFx.push_back({ it->second.x, it->second.y, 0.6f, true });
        s.eggs.erase(it);
    }
}

void EggDeathCommand::execute(GameState& s, std::istringstream& a)
{
    auto it = s.eggs.find(readId(a));
    if (it != s.eggs.end()) {
        s.eggFx.push_back({ it->second.x, it->second.y, 0.6f, false });
        s.eggs.erase(it);
    }
}
