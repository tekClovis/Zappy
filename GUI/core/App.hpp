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
#include "../render/IRenderer.hpp"
#include <memory>
#include <raylib.h>

enum class Phase { Connecting, Running };

class App {
public:
    explicit App(const Args& args);
    ~App();
    void run();

private:
    void processMessages();
    void update(float dt);
    void handleInput();
    float uiScale() const;
    float panelWidth() const;
    Rectangle mapArea() const;
    void renderLoading() const;
    void renderPanel() const;

    Network _net;
    GameState _state;
    Parser _parser;
    std::unique_ptr<IRenderer> _renderer;
    Phase _phase = Phase::Connecting;
    int _selected = -1;
};
