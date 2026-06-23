/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Renderer3D
*/

#pragma once
#include "IRenderer.hpp"

// 3D view: the world is drawn on a flat board, entities as 3D shapes, with an
// orbital camera (right-drag to rotate, wheel to zoom).
class Renderer3D : public IRenderer {
public:
    void update(const GameState& state, float dt) override;
    void draw(const GameState& state, Rectangle area, int selected) override;
    int pickPlayer(const GameState& state, Rectangle area,
                   Vector2 mouse) override;

private:
    void ensureInit(const GameState& state);
    Camera3D camera() const;

    bool _init = false;
    Vector3 _target{};
    float _dist = 20.0f;
    float _yaw = -0.7f;   // radians, around the vertical axis
    float _pitch = 0.9f;  // radians, above the horizon
};
