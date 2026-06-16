/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** App
*/

#include "App.hpp"
#include <raylib.h>
#include <algorithm>
#include <optional>
#include <string>

static const Color TEAM_COLORS[] = {
    RED, BLUE, YELLOW, PURPLE, ORANGE, PINK, SKYBLUE, LIME
};

App::App(const Args& args) : _parser(_state)
{
    _net.connect(args.host, args.port);
    InitWindow(1280, 720, "Zappy");
    SetTargetFPS(60);
}

App::~App()
{
    CloseWindow();
}

void App::run()
{
    while (!WindowShouldClose()) {
        _net.update();
        processMessages();
        update(GetFrameTime());

        BeginDrawing();
        ClearBackground({ 20, 20, 20, 255 });
        if (_state.width == 0)
            renderLoading();
        else
            renderGame();
        EndDrawing();
    }
}

void App::update(float dt)
{
    // tiles per second the rendered position catches up to the logical one
    const float speed = 6.0f;
    float t = std::min(1.0f, speed * dt);

    for (auto& [id, p] : _state.players) {
        if (!p.spawned) {
            p.renderX = (float)p.x;
            p.renderY = (float)p.y;
            p.spawned = true;
            continue;
        }
        // a single Forward only moves one tile, so a bigger gap means a
        // world wrap-around: snap instead of sliding across the whole map
        if (std::abs((float)p.x - p.renderX) > 1.5f)
            p.renderX = (float)p.x;
        if (std::abs((float)p.y - p.renderY) > 1.5f)
            p.renderY = (float)p.y;
        p.renderX += ((float)p.x - p.renderX) * t;
        p.renderY += ((float)p.y - p.renderY) * t;
    }
}

void App::processMessages()
{
    std::optional<std::string> line;
    while ((line = _net.nextLine())) {
        if (_phase == Phase::Connecting) {
            if (*line == "WELCOME") {
                _net.send("GRAPHIC\n");
                _net.send("msz\n");
                _net.send("mct\n");
                _net.send("tna\n");
                _net.send("sgt\n");
                _phase = Phase::Running;
            }
        } else {
            _parser.parse(*line);
        }
    }
}

void App::renderLoading() const
{
    const char *msg = (_phase == Phase::Connecting) ? "Connecting..." : "Loading map...";
    int w = MeasureText(msg, 30);
    DrawText(msg, (GetScreenWidth() - w) / 2, GetScreenHeight() / 2 - 15, 30, WHITE);
}

void App::renderGame() const
{
    const float margin = 10.0f;
    float availW = (float)GetScreenWidth() - 2.0f * margin;
    float availH = (float)GetScreenHeight() - 2.0f * margin - 30.0f;
    float tileSize = std::min(availW / (float)_state.width, availH / (float)_state.height);
    float ox = margin + (availW - tileSize * (float)_state.width) / 2.0f;
    float oy = margin + (availH - tileSize * (float)_state.height) / 2.0f;

    // tiles
    for (int y = 0; y < _state.height; y++) {
        for (int x = 0; x < _state.width; x++) {
            Rectangle rec = {
                ox + (float)x * tileSize, oy + (float)y * tileSize,
                tileSize - 1.0f, tileSize - 1.0f
            };
            DrawRectangleRec(rec, { 34, 85, 34, 255 });
            // food dot
            if (_state.map[y][x].res[0] > 0)
                DrawRectangle((int)rec.x + 2, (int)rec.y + 2, 4, 4, ORANGE);
        }
    }

    // eggs
    for (auto& [id, egg] : _state.eggs) {
        float ex = ox + (float)egg.x * tileSize + tileSize * 0.5f;
        float ey = oy + (float)egg.y * tileSize + tileSize * 0.5f;
        DrawCircle((int)ex, (int)ey, tileSize * 0.15f, WHITE);
    }

    // players
    for (auto& [id, p] : _state.players) {
        Vector2 center = {
            ox + (p.renderX + 0.5f) * tileSize,
            oy + (p.renderY + 0.5f) * tileSize
        };
        int tidx = 0;
        for (int i = 0; i < (int)_state.teams.size(); i++)
            if (_state.teams[i] == p.team) { tidx = i; break; }
        Color c = TEAM_COLORS[tidx % 8];
        float radius = tileSize * 0.30f;
        // triangle tip points toward the facing direction (1=N 2=E 3=S 4=W)
        float rot = (float)(p.orientation - 2) * 90.0f;

        if (p.incanting)
            DrawCircle((int)center.x, (int)center.y, radius * 1.5f, { 255, 255, 0, 110 });
        DrawPoly(center, 3, radius, rot, c);
        DrawPolyLines(center, 3, radius, rot, BLACK);

        if (tileSize >= 16) {
            std::string lv = std::to_string(p.level);
            int tw = MeasureText(lv.c_str(), 10);
            DrawText(lv.c_str(), (int)(center.x - (float)tw / 2.0f),
                     (int)(center.y - radius - 12.0f), 10, WHITE);
        }
    }

    // HUD
    if (_state.over) {
        std::string msg = "Game Over - Winner: " + _state.winner;
        DrawText(msg.c_str(), 10, GetScreenHeight() - 25, 20, GOLD);
    } else {
        std::string info = "Map: " + std::to_string(_state.width) + "x" +
            std::to_string(_state.height) +
            "  Players: " + std::to_string(_state.players.size()) +
            "  Speed: " + std::to_string(_state.timeUnit);
        DrawText(info.c_str(), 10, GetScreenHeight() - 25, 16, LIGHTGRAY);
    }
}
