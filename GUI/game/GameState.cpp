/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** GameState
*/

#include "GameState.hpp"
#include <algorithm>
#include <cmath>

Player* GameState::findPlayer(int id)
{
    auto it = _players.find(id);
    return it == _players.end() ? nullptr : &it->second;
}

// --- map ---

void GameState::resize(int w, int h)
{
    // the server may resend msz; don't wipe an already-sized map
    if (w == _width && h == _height && !_map.empty())
        return;
    _width = w;
    _height = h;
    _map.assign(h, std::vector<Tile>(w));
}

void GameState::setTileResources(int x, int y, const std::array<int, 7>& res)
{
    if (x >= 0 && x < _width && y >= 0 && y < _height)
        _map[y][x].res = res;
}

// --- players ---

void GameState::addPlayer(const Player& p)
{
    if (p.id >= 0)
        _players[p.id] = p;
}

void GameState::removePlayer(int id)
{
    _players.erase(id);
}

void GameState::movePlayer(int id, int x, int y, int orientation)
{
    if (Player* p = findPlayer(id)) {
        p->x = x;
        p->y = y;
        p->orientation = orientation;
    }
}

void GameState::setPlayerLevel(int id, int level)
{
    if (Player* p = findPlayer(id))
        p->level = level;
}

void GameState::setPlayerInventory(int id, const std::array<int, 7>& inv)
{
    if (Player* p = findPlayer(id))
        p->inventory = inv;
}

void GameState::playerDrop(int id, int resource)
{
    Player* p = findPlayer(id);
    if (p && resource >= 0 && resource < 7)
        _map[p->y][p->x].res[resource]++;
}

void GameState::playerTake(int id, int resource)
{
    Player* p = findPlayer(id);
    if (p && resource >= 0 && resource < 7) {
        int& q = _map[p->y][p->x].res[resource];
        q = std::max(0, q - 1);
    }
}

// --- eggs ---

void GameState::addEgg(const Egg& e)
{
    if (e.id >= 0)
        _eggs[e.id] = e;
}

void GameState::hatchEgg(int id)
{
    auto it = _eggs.find(id);
    if (it != _eggs.end()) {
        _eggFx.push_back({ it->second.x, it->second.y, 0.6f, true });
        _eggs.erase(it);
    }
}

void GameState::killEgg(int id)
{
    auto it = _eggs.find(id);
    if (it != _eggs.end()) {
        _eggFx.push_back({ it->second.x, it->second.y, 0.6f, false });
        _eggs.erase(it);
    }
}

// --- teams / server ---

void GameState::addTeam(const std::string& name)
{
    if (name.empty())
        return;
    // the server may resend tna; keep the team list unique
    if (std::find(_teams.begin(), _teams.end(), name) != _teams.end())
        return;
    _teams.push_back(name);
}

void GameState::pushMessage(const std::string& msg)
{
    _serverMessages.push_back(msg);
    if (_serverMessages.size() > 50)
        _serverMessages.erase(_serverMessages.begin());
}

void GameState::endGame(const std::string& winner)
{
    _over = true;
    _winner = winner;
}

// --- incantations ---

void GameState::startIncantation(int x, int y, int level)
{
    _incantations.push_back({ x, y, level });
}

void GameState::setPlayerIncanting(int id, bool on)
{
    if (Player* p = findPlayer(id))
        p->incanting = on;
}

void GameState::endIncantation(int x, int y, bool success)
{
    std::erase_if(_incantations, [x, y](const Incantation& i) {
        return i.x == x && i.y == y;
    });
    _incantResults.push_back({ x, y, success, 2.5f });
    for (auto& [id, p] : _players)
        if (p.x == x && p.y == y)
            p.incanting = false;
}

// --- transient effects ---

void GameState::addBroadcast(int playerId, const std::string& text)
{
    _broadcasts.push_back({ playerId, text, 3.0f });
}

void GameState::addEject(int playerId)
{
    _ejects.push_back({ playerId, 1.0f });
}

// --- per-frame evolution ---

void GameState::update(float dt)
{
    // entity interpolation. A move is at most one tile, so a gap bigger than
    // that means a world wrap-around: snap to the new side (the player stays
    // inside the map bounds) instead of sliding across/over the border.
    const float speed = 6.0f;
    float t = std::min(1.0f, speed * dt);
    for (auto& [id, p] : _players) {
        if (!p.spawned) {
            p.renderX = (float)p.x;
            p.renderY = (float)p.y;
            p.spawned = true;
            continue;
        }
        if (std::abs((float)p.x - p.renderX) > 1.5f)
            p.renderX = (float)p.x;
        if (std::abs((float)p.y - p.renderY) > 1.5f)
            p.renderY = (float)p.y;
        p.renderX += ((float)p.x - p.renderX) * t;
        p.renderY += ((float)p.y - p.renderY) * t;
    }

    // fade out transient effects
    for (auto& r : _incantResults)
        r.timer -= dt;
    std::erase_if(_incantResults,
                  [](const IncantationResult& r) { return r.timer <= 0.0f; });
    for (auto& b : _broadcasts)
        b.timer -= dt;
    std::erase_if(_broadcasts,
                  [](const Broadcast& b) { return b.timer <= 0.0f; });
    for (auto& e : _ejects)
        e.timer -= dt;
    std::erase_if(_ejects, [](const EjectFx& e) { return e.timer <= 0.0f; });

    // eggs: age (pop-in) + hatch/death bursts
    for (auto& [id, egg] : _eggs)
        egg.age += dt;
    for (auto& f : _eggFx)
        f.timer -= dt;
    std::erase_if(_eggFx, [](const EggFx& f) { return f.timer <= 0.0f; });
}
