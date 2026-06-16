/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** App
*/

#pragma once
#include "Args.hpp"
#include "Network.hpp"
#include "../game/GameState.hpp"
#include "../protocol/Parser.hpp"

enum class Phase { Connecting, Running };

class App {
public:
    explicit App(const Args& args);
    ~App();
    void run();

private:
    void processMessages();
    void renderLoading() const;
    void renderGame() const;

    Network _net;
    GameState _state;
    Parser _parser;
    Phase _phase = Phase::Connecting;
};
