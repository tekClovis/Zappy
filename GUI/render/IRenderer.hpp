/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** IRenderer
*/

#pragma once
#include "../game/GameState.hpp"
#include <raylib.h>

// A renderer draws the WORLD (map, entities, effects) inside a screen area.
// The HUD/info panel overlay stays in App and is always 2D.
class IRenderer {
public:
    virtual ~IRenderer() = default;

    // per-frame camera/animation update (entity interpolation stays in App)
    virtual void update(const GameState& state, float dt) = 0;

    // draw the world inside `area` (the screen rect left of the info panel)
    virtual void draw(const GameState& state, Rectangle area, int selected) = 0;

    // map a mouse position to the player id under the cursor, or -1
    virtual int pickPlayer(const GameState& state, Rectangle area,
                           Vector2 mouse) = 0;
};
