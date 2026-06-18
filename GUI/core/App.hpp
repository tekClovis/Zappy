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
    struct Layout {
        float tileSize;
        float ox;
        float oy;
    };

    void processMessages();
    void update(float dt);
    void handleInput();
    Layout computeLayout() const;
    void renderLoading() const;
    void renderGame() const;
    void renderPanel() const;

    Network _net;
    GameState _state;
    Parser _parser;
    Phase _phase = Phase::Connecting;
    int _selected = -1;
};
