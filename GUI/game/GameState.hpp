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
};

struct GameState {
    int width{}, height{};
    std::vector<std::vector<Tile>> map;
    std::map<int, Player> players;
    std::map<int, Egg> eggs;
    std::vector<std::string> teams;
    int timeUnit{ 100 };
    bool over{};
    std::string winner;

    void resize(int w, int h);
    Tile& tile(int x, int y) { return map[y][x]; }
    const Tile& tile(int x, int y) const { return map[y][x]; }
};
