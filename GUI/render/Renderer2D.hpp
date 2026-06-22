/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Renderer2D
*/

#pragma once
#include "IRenderer.hpp"

// Top-down 2D view: the map fits inside the given area, entities and effects
// are drawn as flat shapes.
class Renderer2D : public IRenderer {
public:
    void update(const GameState& state, float dt) override;
    void draw(const GameState& state, Rectangle area, int selected) override;
    int pickPlayer(const GameState& state, Rectangle area,
                   Vector2 mouse) override;

private:
    struct Layout {
        float tileSize;
        float ox;
        float oy;
    };

    Layout computeLayout(const GameState& state, Rectangle area) const;
};
