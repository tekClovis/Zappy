/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** App
*/

#include "App.hpp"
#include "../render/Renderer2D.hpp"
#include "../render/Renderer3D.hpp"
#include "../render/Palette.hpp"
#include <raylib.h>
#include <algorithm>
#include <memory>
#include <optional>
#include <string>

static const float PANEL_W = 300.0f;
static const float MARGIN = 10.0f;

App::App(const Args& args) : _dispatcher(_state)
{
    _net.connect(args.host, args.port);
    SetConfigFlags(FLAG_WINDOW_RESIZABLE);
    InitWindow(1280, 720, "Zappy");
    SetWindowMinSize(800, 600);
    SetTargetFPS(60);
    _renderer = std::make_unique<Renderer2D>();
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
            _renderer->draw(_state, mapArea(), _selected);
            renderPanel();
        }
        if (!_net.isConnected()) {
            const char *m = "Connection lost";
            int fs = 30;
            int w = MeasureText(m, fs);
            DrawRectangle(0, GetScreenHeight() / 2 - 30,
                          GetScreenWidth(), 60, { 0, 0, 0, 200 });
            DrawText(m, (GetScreenWidth() - w) / 2,
                     GetScreenHeight() / 2 - 15, fs, RED);
        }
        EndDrawing();
    }
}

void App::update(float dt)
{
    // tiles per second the rendered position catches up to the logical one
    const float speed = 6.0f;
    float t = std::min(1.0f, speed * dt);

    float w = (float)_state.width;
    float h = (float)_state.height;
    for (auto& [id, p] : _state.players) {
        if (!p.spawned) {
            p.renderX = (float)p.x;
            p.renderY = (float)p.y;
            p.spawned = true;
            continue;
        }
        // shortest toroidal path so crossing an edge slides out one side
        // and back in from the other instead of teleporting across the map
        float dx = (float)p.x - p.renderX;
        float dy = (float)p.y - p.renderY;
        if (dx > w / 2.0f) dx -= w;
        if (dx < -w / 2.0f) dx += w;
        if (dy > h / 2.0f) dy -= h;
        if (dy < -h / 2.0f) dy += h;
        p.renderX += dx * t;
        p.renderY += dy * t;
        if (p.renderX < 0.0f) p.renderX += w;
        if (p.renderX >= w) p.renderX -= w;
        if (p.renderY < 0.0f) p.renderY += h;
        if (p.renderY >= h) p.renderY -= h;
    }

    // fade out the incantation result flashes
    for (auto& r : _state.incantResults)
        r.timer -= dt;
    std::erase_if(_state.incantResults,
                  [](const IncantationResult& r) { return r.timer <= 0.0f; });

    // fade out broadcast waves and ejection bursts
    for (auto& b : _state.broadcasts)
        b.timer -= dt;
    std::erase_if(_state.broadcasts,
                  [](const Broadcast& b) { return b.timer <= 0.0f; });
    for (auto& e : _state.ejects)
        e.timer -= dt;
    std::erase_if(_state.ejects,
                  [](const EjectFx& e) { return e.timer <= 0.0f; });

    // eggs: age (pop-in) + hatch/death bursts
    for (auto& [id, egg] : _state.eggs)
        egg.age += dt;
    for (auto& f : _state.eggFx)
        f.timer -= dt;
    std::erase_if(_state.eggFx,
                  [](const EggFx& f) { return f.timer <= 0.0f; });

    _renderer->update(_state, dt);
}

float App::uiScale() const
{
    // HUD scales with window height, baseline design is 720p
    return (float)GetScreenHeight() / 720.0f;
}

float App::panelWidth() const
{
    return PANEL_W * uiScale();
}

Rectangle App::mapArea() const
{
    // the world area is the screen minus the right-side info panel
    return {
        MARGIN, MARGIN,
        (float)GetScreenWidth() - panelWidth() - 2.0f * MARGIN,
        (float)GetScreenHeight() - 2.0f * MARGIN
    };
}

void App::handleInput()
{
    if (IsKeyPressed(KEY_F11))
        ToggleBorderlessWindowed();

    // toggle 2D / 3D view
    if (IsKeyPressed(KEY_TAB)) {
        _mode3D = !_mode3D;
        if (_mode3D)
            _renderer = std::make_unique<Renderer3D>();
        else
            _renderer = std::make_unique<Renderer2D>();
    }

    // speed control: ask the server to change the time unit (sst)
    if (_net.isConnected() && _state.width != 0) {
        if (IsKeyPressed(KEY_EQUAL) || IsKeyPressed(KEY_KP_ADD))
            _net.send("sst " + std::to_string(_state.timeUnit + 10) + "\n");
        if (IsKeyPressed(KEY_MINUS) || IsKeyPressed(KEY_KP_SUBTRACT)) {
            int nt = _state.timeUnit - 10;
            if (nt < 1)
                nt = 1;
            _net.send("sst " + std::to_string(nt) + "\n");
        }
    }

    if (_state.width == 0 || !IsMouseButtonPressed(MOUSE_BUTTON_LEFT))
        return;
    int id = _renderer->pickPlayer(_state, mapArea(), GetMousePosition());
    _selected = id;
    if (id >= 0)
        _net.send("pin #" + std::to_string(id) + "\n");
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
            _dispatcher.dispatch(*line);
        }
    }
}

void App::renderLoading() const
{
    const char *msg = (_phase == Phase::Connecting) ? "Connecting..." : "Loading map...";
    int w = MeasureText(msg, 30);
    DrawText(msg, (GetScreenWidth() - w) / 2, GetScreenHeight() / 2 - 15, 30, WHITE);
}

void App::renderPanel() const
{
    float s = uiScale();
    float pw = panelWidth();
    float px = (float)GetScreenWidth() - pw;
    DrawRectangle((int)px, 0, (int)pw + 1, GetScreenHeight(), { 30, 30, 38, 255 });
    DrawLine((int)px, 0, (int)px, GetScreenHeight(), { 60, 60, 70, 255 });

    int tx = (int)px + (int)(15 * s);
    int right = (int)px + (int)pw - (int)(15 * s);
    int hf = (int)(16 * s);     // standard HUD font
    float hl = 22 * s;          // standard line height
    float y = 15 * s;

    DrawText("ZAPPY", tx, (int)y, (int)(26 * s), WHITE);
    y += 40 * s;

    // global HUD
    DrawText(TextFormat("Map: %d x %d", _state.width, _state.height),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    DrawText(TextFormat("Players: %d", (int)_state.players.size()),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    DrawText(TextFormat("Eggs: %d", (int)_state.eggs.size()),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    DrawText(TextFormat("Time unit: %d  [-/+]", _state.timeUnit),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    DrawText(TextFormat("Teams: %d", (int)_state.teams.size()),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    for (int i = 0; i < (int)_state.teams.size(); i++) {
        int cnt = 0;
        for (const auto& [id, p] : _state.players)
            if (p.team == _state.teams[i]) cnt++;
        DrawCircle(tx + (int)(6 * s), (int)y + (int)(7 * s), 5.0f * s,
                   TEAM_COLORS[i % 8]);
        DrawText(TextFormat("%s (%d)", _state.teams[i].c_str(), cnt),
                 tx + (int)(16 * s), (int)y, (int)(14 * s), LIGHTGRAY);
        y += 20 * s;
    }
    y += 10 * s;

    DrawLine(tx, (int)y, right, (int)y, { 60, 60, 70, 255 });
    y += 12 * s;

    // selected player detail
    auto it = _state.players.find(_selected);
    if (it == _state.players.end()) {
        DrawText("Click a player", tx, (int)y, hf, GRAY);
    } else {
        const Player& p = it->second;
        DrawText(TextFormat("Player #%d", p.id), tx, (int)y, (int)(20 * s), WHITE);
        y += 28 * s;
        DrawText(TextFormat("Team: %s", p.team.c_str()), tx, (int)y, hf, LIGHTGRAY); y += hl;
        DrawText(TextFormat("Level: %d", p.level), tx, (int)y, hf, LIGHTGRAY); y += hl;
        // 1 food unit = 126 time units of life; seconds depend on the time unit
        float lifeSec = _state.timeUnit > 0
            ? (float)p.inventory[0] * 126.0f / (float)_state.timeUnit : 0.0f;
        DrawText(TextFormat("Food: %d  (~%.0fs)", p.inventory[0], lifeSec),
                 tx, (int)y, hf, LIGHTGRAY); y += hl;
        DrawText(TextFormat("Pos: (%d, %d)", p.x, p.y), tx, (int)y, hf, LIGHTGRAY); y += hl;
        const char *dir[] = { "?", "North", "East", "South", "West" };
        int o = (p.orientation >= 1 && p.orientation <= 4) ? p.orientation : 0;
        DrawText(TextFormat("Facing: %s", dir[o]), tx, (int)y, hf, LIGHTGRAY); y += 28 * s;

        DrawText("Inventory:", tx, (int)y, hf, WHITE); y += hl;
        for (int i = 0; i < 7; i++) {
            DrawText(TextFormat("  %-10s %d", RES_NAMES[i], p.inventory[i]),
                     tx, (int)y, (int)(15 * s), LIGHTGRAY);
            y += 19 * s;
        }
    }

    // server messages at the bottom
    if (!_state.serverMessages.empty()) {
        float lineH = 19 * s;
        int logY = GetScreenHeight() - (int)(15 * s) - (int)(lineH * 6);
        DrawLine(tx, logY - (int)(10 * s), right, logY - (int)(10 * s),
                 { 60, 60, 70, 255 });
        DrawText("Server log:", tx, logY - (int)(28 * s), hf, WHITE);
        int n = (int)_state.serverMessages.size();
        int start = n > 6 ? n - 6 : 0;
        float ly = (float)logY;
        for (int i = start; i < n; i++) {
            DrawText(_state.serverMessages[i].c_str(), tx, (int)ly, (int)(13 * s), GRAY);
            ly += lineH;
        }
    }

    // game over banner
    if (_state.over) {
        DrawRectangle(0, GetScreenHeight() / 2 - (int)(40 * s),
                      GetScreenWidth(), (int)(80 * s), { 0, 0, 0, 200 });
        std::string msg = "GAME OVER - Winner: " + _state.winner;
        int fs = (int)(30 * s);
        int w = MeasureText(msg.c_str(), fs);
        DrawText(msg.c_str(), (GetScreenWidth() - w) / 2,
                 GetScreenHeight() / 2 - (int)(15 * s), fs, GOLD);
    }
}
