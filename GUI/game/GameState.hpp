/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** GameState
*/

#pragma once
#include <array>
#include <map>
#include <string>
#include <vector>

// --- value records (plain data carried by the state) ---

struct Tile {
    std::array<int, 7> res{};
    // indices: 0=food 1=linemate 2=deraumere 3=sibur 4=mendiane 5=phiras 6=thystame
};

struct Player {
    int id{};
    int x{}, y{};
    int orientation{}; // 1=N 2=E 3=S 4=W
    int level{};
    std::string team;
    std::array<int, 7> inventory{};
    bool incanting{};
    // render-only: interpolated position for smooth movement between tiles
    float renderX{};
    float renderY{};
    bool spawned{};
};

struct Egg {
    int id{};
    int playerId{};
    int x{}, y{};
    float age{}; // render-only: seconds since spawn, for the pop-in animation
};

struct Incantation {
    int x{}, y{};
    int level{};
};

struct IncantationResult {
    int x{}, y{};
    bool success{};
    float timer{};
};

struct Broadcast {
    int playerId{};
    std::string text;
    float timer{};
};

struct EjectFx {
    int playerId{};
    float timer{};
};

struct EggFx {
    int x{}, y{};
    float timer{};
    bool hatched{};
};

// Aggregate of the whole world. Owns its data (private) and exposes intent-
// revealing operations; the protocol commands mutate it through these methods
// and the renderers read it through the const accessors.
class GameState {
public:
    // --- map / dimensions ---
    int width() const { return _width; }
    int height() const { return _height; }
    void resize(int w, int h);
    const Tile& tile(int x, int y) const { return _map[y][x]; }
    void setTileResources(int x, int y, const std::array<int, 7>& res);

    // --- players ---
    const std::map<int, Player>& players() const { return _players; }
    void addPlayer(const Player& p);
    void removePlayer(int id);
    void movePlayer(int id, int x, int y, int orientation);
    void setPlayerLevel(int id, int level);
    void setPlayerInventory(int id, const std::array<int, 7>& inv);
    void playerDrop(int id, int resource);
    void playerTake(int id, int resource);

    // --- eggs ---
    const std::map<int, Egg>& eggs() const { return _eggs; }
    void addEgg(const Egg& e);
    void hatchEgg(int id);
    void killEgg(int id);

    // --- teams / server ---
    const std::vector<std::string>& teams() const { return _teams; }
    void addTeam(const std::string& name);
    const std::vector<std::string>& serverMessages() const { return _serverMessages; }
    void pushMessage(const std::string& msg);
    int timeUnit() const { return _timeUnit; }
    void setTimeUnit(int t) { _timeUnit = t; }
    bool isOver() const { return _over; }
    const std::string& winner() const { return _winner; }
    void endGame(const std::string& winner);

    // --- incantations ---
    const std::vector<Incantation>& incantations() const { return _incantations; }
    const std::vector<IncantationResult>& incantResults() const { return _incantResults; }
    void startIncantation(int x, int y, int level);
    void setPlayerIncanting(int id, bool on);
    void endIncantation(int x, int y, bool success);

    // --- transient effects ---
    const std::vector<Broadcast>& broadcasts() const { return _broadcasts; }
    const std::vector<EjectFx>& ejects() const { return _ejects; }
    const std::vector<EggFx>& eggFx() const { return _eggFx; }
    void addBroadcast(int playerId, const std::string& text);
    void addEject(int playerId);

    // per-frame evolution: entity interpolation + effect timers + egg age
    void update(float dt);

private:
    Player* findPlayer(int id);

    int _width = 0;
    int _height = 0;
    std::vector<std::vector<Tile>> _map;
    std::map<int, Player> _players;
    std::map<int, Egg> _eggs;
    std::vector<std::string> _teams;
    std::vector<std::string> _serverMessages;
    std::vector<Incantation> _incantations;
    std::vector<IncantationResult> _incantResults;
    std::vector<Broadcast> _broadcasts;
    std::vector<EjectFx> _ejects;
    std::vector<EggFx> _eggFx;
    int _timeUnit = 100;
    bool _over = false;
    std::string _winner;
};
