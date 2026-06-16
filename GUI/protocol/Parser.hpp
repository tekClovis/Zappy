/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Parser
*/

#pragma once
#include "../game/GameState.hpp"
#include <string>

class Parser {
public:
    explicit Parser(GameState& state) : _s(state) {}
    void parse(const std::string& line);

private:
    GameState& _s;
};
