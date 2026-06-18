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

static const char *RES_NAMES[] = {
    "food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame"
};

static const float PANEL_W = 300.0f;
static const float MARGIN = 10.0f;

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
        handleInput();

        BeginDrawing();
        ClearBackground({ 20, 20, 20, 255 });
        if (_state.width == 0) {
            renderLoading();
        } else {
            renderGame();
            renderPanel();
        }
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

App::Layout App::computeLayout() const
{
    // map area is the screen minus the right-side info panel
    float availW = (float)GetScreenWidth() - PANEL_W - 2.0f * MARGIN;
    float availH = (float)GetScreenHeight() - 2.0f * MARGIN;
    float tileSize = std::min(availW / (float)_state.width,
                              availH / (float)_state.height);
    return {
        tileSize,
        MARGIN + (availW - tileSize * (float)_state.width) / 2.0f,
        MARGIN + (availH - tileSize * (float)_state.height) / 2.0f
    };
}

void App::handleInput()
{
    if (_state.width == 0 || !IsMouseButtonPressed(MOUSE_BUTTON_LEFT))
        return;

    Layout l = computeLayout();
    Vector2 m = GetMousePosition();
    int tx = (int)((m.x - l.ox) / l.tileSize);
    int ty = (int)((m.y - l.oy) / l.tileSize);

    _selected = -1;
    if (tx < 0 || tx >= _state.width || ty < 0 || ty >= _state.height)
        return;
    for (auto& [id, p] : _state.players) {
        if (p.x == tx && p.y == ty) {
            _selected = id;
            // ask the server for a fresh inventory of the selected player
            _net.send("pin #" + std::to_string(id) + "\n");
            break;
        }
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
    Layout l = computeLayout();
    float tileSize = l.tileSize;
    float ox = l.ox;
    float oy = l.oy;

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

        if (id == _selected)
            DrawCircleLines((int)center.x, (int)center.y, radius * 1.7f, WHITE);
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

}

void App::renderPanel() const
{
    float px = (float)GetScreenWidth() - PANEL_W;
    DrawRectangle((int)px, 0, (int)PANEL_W, GetScreenHeight(), { 30, 30, 38, 255 });
    DrawLine((int)px, 0, (int)px, GetScreenHeight(), { 60, 60, 70, 255 });

    int tx = (int)px + 15;
    int y = 15;

    DrawText("ZAPPY", tx, y, 26, WHITE);
    y += 40;

    // global HUD
    DrawText(TextFormat("Map: %d x %d", _state.width, _state.height),
             tx, y, 16, LIGHTGRAY); y += 22;
    DrawText(TextFormat("Players: %d", (int)_state.players.size()),
             tx, y, 16, LIGHTGRAY); y += 22;
    DrawText(TextFormat("Eggs: %d", (int)_state.eggs.size()),
             tx, y, 16, LIGHTGRAY); y += 22;
    DrawText(TextFormat("Time unit: %d", _state.timeUnit),
             tx, y, 16, LIGHTGRAY); y += 22;
    DrawText(TextFormat("Teams: %d", (int)_state.teams.size()),
             tx, y, 16, LIGHTGRAY); y += 30;

    DrawLine(tx, y, (int)px + (int)PANEL_W - 15, y, { 60, 60, 70, 255 });
    y += 12;

    // selected player detail
    auto it = _state.players.find(_selected);
    if (it == _state.players.end()) {
        DrawText("Click a player", tx, y, 16, GRAY);
    } else {
        const Player& p = it->second;
        DrawText(TextFormat("Player #%d", p.id), tx, y, 20, WHITE); y += 28;
        DrawText(TextFormat("Team: %s", p.team.c_str()), tx, y, 16, LIGHTGRAY); y += 22;
        DrawText(TextFormat("Level: %d", p.level), tx, y, 16, LIGHTGRAY); y += 22;
        DrawText(TextFormat("Pos: (%d, %d)", p.x, p.y), tx, y, 16, LIGHTGRAY); y += 22;
        const char *dir[] = { "?", "North", "East", "South", "West" };
        int o = (p.orientation >= 1 && p.orientation <= 4) ? p.orientation : 0;
        DrawText(TextFormat("Facing: %s", dir[o]), tx, y, 16, LIGHTGRAY); y += 28;

        DrawText("Inventory:", tx, y, 16, WHITE); y += 22;
        for (int i = 0; i < 7; i++) {
            DrawText(TextFormat("  %-10s %d", RES_NAMES[i], p.inventory[i]),
                     tx, y, 15, LIGHTGRAY);
            y += 19;
        }
    }

    // server messages at the bottom
    if (!_state.serverMessages.empty()) {
        int logY = GetScreenHeight() - 15 - 19 * 6;
        DrawLine(tx, logY - 10, (int)px + (int)PANEL_W - 15, logY - 10,
                 { 60, 60, 70, 255 });
        DrawText("Server log:", tx, logY - 28, 16, WHITE);
        int n = (int)_state.serverMessages.size();
        int start = n > 6 ? n - 6 : 0;
        for (int i = start; i < n; i++) {
            DrawText(_state.serverMessages[i].c_str(), tx, logY, 13, GRAY);
            logY += 19;
        }
    }

    // game over banner
    if (_state.over) {
        DrawRectangle(0, GetScreenHeight() / 2 - 40, GetScreenWidth(), 80,
                      { 0, 0, 0, 200 });
        std::string msg = "GAME OVER - Winner: " + _state.winner;
        int w = MeasureText(msg.c_str(), 30);
        DrawText(msg.c_str(), (GetScreenWidth() - w) / 2,
                 GetScreenHeight() / 2 - 15, 30, GOLD);
    }
}
