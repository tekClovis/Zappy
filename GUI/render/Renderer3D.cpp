/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Renderer3D
*/

#include "Renderer3D.hpp"
#include "Palette.hpp"
#include <algorithm>
#include <cmath>
#include <string>

static int teamIndex(const GameState& state, const std::string& team)
{
    const auto& teams = state.teams();
    for (int i = 0; i < (int)teams.size(); i++)
        if (teams[i] == team)
            return i;
    return 0;
}

void Renderer3D::ensureInit(const GameState& state)
{
    if (_init || state.width() == 0)
        return;
    _target = { (float)state.width() / 2.0f, 0.0f, (float)state.height() / 2.0f };
    _dist = (float)std::max(state.width(), state.height()) * 1.4f + 5.0f;
    _init = true;
}

Camera3D Renderer3D::camera() const
{
    Camera3D cam{};
    cam.position = {
        (float)(_target.x + _dist * std::cos(_pitch) * std::sin(_yaw)),
        (float)(_target.y + _dist * std::sin(_pitch)),
        (float)(_target.z + _dist * std::cos(_pitch) * std::cos(_yaw))
    };
    cam.target = _target;
    cam.up = { 0.0f, 1.0f, 0.0f };
    cam.fovy = 45.0f;
    cam.projection = CAMERA_PERSPECTIVE;
    return cam;
}

void Renderer3D::update(const GameState& state, float dt)
{
    ensureInit(state);

    // zoom (wheel)
    float wheel = GetMouseWheelMove();
    if (wheel != 0.0f) {
        _dist -= wheel * 2.0f;
        _dist = std::clamp(_dist, 3.0f, 250.0f);
    }

    // rotate (right-drag)
    if (IsMouseButtonDown(MOUSE_BUTTON_RIGHT)) {
        Vector2 d = GetMouseDelta();
        _yaw -= d.x * 0.005f;
        _pitch += d.y * 0.005f;
        _pitch = std::clamp(_pitch, 0.2f, 1.5f);
    }

    // pan along the ground plane (middle-drag or arrow keys)
    float rx = (float)std::cos(_yaw);
    float rz = -(float)std::sin(_yaw);
    float fx = -(float)std::sin(_yaw);
    float fz = -(float)std::cos(_yaw);
    if (IsMouseButtonDown(MOUSE_BUTTON_MIDDLE)) {
        Vector2 d = GetMouseDelta();
        float sp = _dist * 0.0015f;
        _target.x += (-rx * d.x + fx * d.y) * sp;
        _target.z += (-rz * d.x + fz * d.y) * sp;
    }
    float ks = _dist * 0.8f * dt;
    if (IsKeyDown(KEY_LEFT))  { _target.x -= rx * ks; _target.z -= rz * ks; }
    if (IsKeyDown(KEY_RIGHT)) { _target.x += rx * ks; _target.z += rz * ks; }
    if (IsKeyDown(KEY_UP))    { _target.x += fx * ks; _target.z += fz * ks; }
    if (IsKeyDown(KEY_DOWN))  { _target.x -= fx * ks; _target.z -= fz * ks; }

    _target.x = std::clamp(_target.x, 0.0f, (float)state.width());
    _target.z = std::clamp(_target.z, 0.0f, (float)state.height());

    // reset camera (R)
    if (IsKeyPressed(KEY_R)) {
        _yaw = -0.7f;
        _pitch = 0.9f;
        _init = false;
        ensureInit(state);
    }
}

void Renderer3D::draw(const GameState& state, Rectangle area, int selected)
{
    Camera3D cam = camera();

    BeginScissorMode((int)area.x, (int)area.y, (int)area.width, (int)area.height);
    BeginMode3D(cam);

    // ground tiles (checkerboard) + resources as small cubes
    for (int y = 0; y < state.height(); y++) {
        for (int x = 0; x < state.width(); x++) {
            Vector3 pos = { (float)x + 0.5f, 0.0f, (float)y + 0.5f };
            Color base = ((x + y) % 2 == 0)
                ? Color{ 34, 85, 34, 255 } : Color{ 42, 99, 42, 255 };
            DrawCube(pos, 1.0f, 0.2f, 1.0f, base);

            const Tile& t = state.tile(x, y);
            int slot = 0;
            for (int i = 0; i < 7; i++) {
                if (t.res[i] <= 0)
                    continue;
                float rx = (float)x + 0.30f + 0.20f * (float)(slot % 3);
                float rz = (float)y + 0.30f + 0.20f * (float)(slot / 3);
                DrawCube({ rx, 0.20f, rz }, 0.12f, 0.12f, 0.12f, RES_COLORS[i]);
                slot++;
            }
        }
    }

    // eggs (pop-in spheres)
    for (const auto& [id, egg] : state.eggs()) {
        float pop = std::min(1.0f, egg.age / 0.4f);
        DrawSphere({ (float)egg.x + 0.5f, 0.25f, (float)egg.y + 0.5f },
                   0.18f * pop, WHITE);
    }

    // players: oriented cubes (opaque, drawn before the translucent effects so
    // the auras blend over them instead of writing depth and hiding them)
    for (const auto& [id, p] : state.players()) {
        Color c = TEAM_COLORS[teamIndex(state, p.team) % 8];
        Vector3 pos = { p.renderX + 0.5f, 0.35f, p.renderY + 0.5f };

        DrawCube(pos, 0.5f, 0.5f, 0.5f, c);
        DrawCubeWires(pos, 0.5f, 0.5f, 0.5f, BLACK);
        if (id == selected)
            DrawCubeWires(pos, 0.64f, 0.64f, 0.64f, WHITE);

        // facing nose (1=N -z, 2=E +x, 3=S +z, 4=W -x)
        float nx = 0.0f;
        float nz = 0.0f;
        if (p.orientation == 1) nz = -0.35f;
        else if (p.orientation == 2) nx = 0.35f;
        else if (p.orientation == 3) nz = 0.35f;
        else if (p.orientation == 4) nx = -0.35f;
        DrawCube({ pos.x + nx, pos.y, pos.z + nz }, 0.18f, 0.18f, 0.18f, BLACK);
    }

    // translucent effects last (after all opaque geometry)
    for (const auto& inc : state.incantations())
        DrawSphere({ (float)inc.x + 0.5f, 0.3f, (float)inc.y + 0.5f },
                   0.55f, { 255, 215, 0, 90 });
    for (const auto& [id, p] : state.players())
        if (p.incanting)
            DrawSphere({ p.renderX + 0.5f, 0.35f, p.renderY + 0.5f },
                       0.5f, { 255, 255, 0, 90 });

    EndMode3D();
    EndScissorMode();
}

int Renderer3D::pickPlayer(const GameState& state, Rectangle area, Vector2 mouse)
{
    if (!CheckCollisionPointRec(mouse, area))
        return -1;

    Camera3D cam = camera();
    Ray ray = GetMouseRay(mouse, cam);
    int best = -1;
    float bestDist = 1e30f;

    for (const auto& [id, p] : state.players()) {
        Vector3 pos = { p.renderX + 0.5f, 0.35f, p.renderY + 0.5f };
        BoundingBox box = {
            { pos.x - 0.3f, pos.y - 0.3f, pos.z - 0.3f },
            { pos.x + 0.3f, pos.y + 0.3f, pos.z + 0.3f }
        };
        RayCollision col = GetRayCollisionBox(ray, box);
        if (col.hit && col.distance < bestDist) {
            bestDist = col.distance;
            best = id;
        }
    }
    return best;
}
