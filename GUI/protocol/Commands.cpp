/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Commands
*/

#include "Commands.hpp"
#include "ProtocolUtil.hpp"

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
    s.setTileResources(x, y, r);
}

// --- teams / server ---

void TeamNameCommand::execute(GameState& s, std::istringstream& a)
{
    std::string name;
    a >> name;
    s.addTeam(name);
}

void TimeUnitGetCommand::execute(GameState& s, std::istringstream& a)
{
    int t = 0;
    a >> t;
    s.setTimeUnit(t);
}

void TimeUnitSetCommand::execute(GameState& s, std::istringstream& a)
{
    int t = 0;
    a >> t;
    s.setTimeUnit(t);
}

void EndGameCommand::execute(GameState& s, std::istringstream& a)
{
    s.endGame(readRest(a));
}

void ServerMessageCommand::execute(GameState& s, std::istringstream& a)
{
    std::string msg = readRest(a);
    if (!msg.empty())
        s.pushMessage(msg);
}

void UnknownCommand::execute(GameState& s, std::istringstream&)
{
    s.pushMessage("[server] unknown command (suc)");
}

void BadParameterCommand::execute(GameState& s, std::istringstream&)
{
    s.pushMessage("[server] bad parameter (sbp)");
}

// --- players ---

void NewPlayerCommand::execute(GameState& s, std::istringstream& a)
{
    Player p;
    p.id = readId(a);
    a >> p.x >> p.y >> p.orientation >> p.level >> p.team;
    s.addPlayer(p);
}

void PlayerPositionCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int x = 0;
    int y = 0;
    int o = 0;
    a >> x >> y >> o;
    s.movePlayer(id, x, y, o);
}

void PlayerLevelCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int level = 0;
    a >> level;
    s.setPlayerLevel(id, level);
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
    s.setPlayerInventory(id, r);
}

void PlayerDeathCommand::execute(GameState& s, std::istringstream& a)
{
    s.removePlayer(readId(a));
}

void IncantationStartCommand::execute(GameState& s, std::istringstream& a)
{
    int x = 0;
    int y = 0;
    int level = 0;
    a >> x >> y >> level;
    s.startIncantation(x, y, level);
    int id = -1;
    while ((id = readId(a)) >= 0)
        s.setPlayerIncanting(id, true);
}

void IncantationEndCommand::execute(GameState& s, std::istringstream& a)
{
    int x = 0;
    int y = 0;
    int result = 0;
    a >> x >> y >> result;
    s.endIncantation(x, y, result != 0);
}

void PlayerDropCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int i = -1;
    a >> i;
    s.playerDrop(id, i);
}

void PlayerTakeCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    int i = -1;
    a >> i;
    s.playerTake(id, i);
}

void BroadcastCommand::execute(GameState& s, std::istringstream& a)
{
    int id = readId(a);
    s.addBroadcast(id, readRest(a));
}

void EjectCommand::execute(GameState& s, std::istringstream& a)
{
    s.addEject(readId(a));
}

// --- eggs ---

void EggLaidCommand::execute(GameState& s, std::istringstream& a)
{
    Egg e;
    e.id = readId(a);
    e.playerId = readId(a);
    a >> e.x >> e.y;
    s.addEgg(e);
}

void EggHatchCommand::execute(GameState& s, std::istringstream& a)
{
    s.hatchEgg(readId(a));
}

void EggDeathCommand::execute(GameState& s, std::istringstream& a)
{
    s.killEgg(readId(a));
}
