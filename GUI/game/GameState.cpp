/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** GameState
*/

#include "GameState.hpp"

void GameState::resize(int w, int h)
{
    width = w;
    height = h;
    map.assign(h, std::vector<Tile>(w));
}
