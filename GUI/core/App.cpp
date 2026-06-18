/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** App
*/

#include "App.hpp"
#include <raylib.h>
#include <algorithm>
#include <cmath>
#include <optional>
#include <string>

static const Color TEAM_COLORS[] = {
    RED, BLUE, YELLOW, PURPLE, ORANGE, PINK, SKYBLUE, LIME
};

static const char *RES_NAMES[] = {
    "food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame"
};

// same index order as RES_NAMES
static const Color RES_COLORS[] = {
    GREEN, RAYWHITE, BROWN, DARKBLUE, VIOLET, MAROON, GOLD
};

static const float PANEL_W = 300.0f;
static const float MARGIN = 10.0f;

static void drawTileResources(const Tile& t, float tx, float ty, float size)
{
    float pad = size * 0.12f;
    float cell = (size - 2.0f * pad) / 3.0f;
    float pip = cell * 0.30f;
    if (pip < 1.5f) pip = 1.5f;

    for (int i = 0; i < 7; i++) {
        if (t.res[i] <= 0)
            continue;
        float cx = tx + pad + cell * ((float)(i % 3) + 0.5f);
        float cy = ty + pad + cell * ((float)(i / 3) + 0.5f);
        DrawCircle((int)cx, (int)cy, pip, RES_COLORS[i]);
        if (size >= 46)
            DrawText(TextFormat("%d", t.res[i]),
                     (int)(cx + pip + 1.0f), (int)(cy - pip), 10, RAYWHITE);
    }
}

static void drawLegend(float s)
{
    int x = (int)(14 * s);
    int y = (int)(14 * s);
    int lh = (int)(16 * s);
    int fs = (int)(12 * s);
    DrawRectangle(x - (int)(6 * s), y - (int)(6 * s),
                  (int)(120 * s), 7 * lh + (int)(12 * s), { 0, 0, 0, 150 });
    for (int i = 0; i < 7; i++) {
        DrawCircle(x + (int)(5 * s), y + (int)(8 * s) + i * lh, 5.0f * s, RES_COLORS[i]);
        DrawText(RES_NAMES[i], x + (int)(16 * s), y + (int)(2 * s) + i * lh, fs, RAYWHITE);
    }
}

App::App(const Args& args) : _parser(_state)
{
    _net.connect(args.host, args.port);
    SetConfigFlags(FLAG_WINDOW_RESIZABLE);
    InitWindow(1280, 720, "Zappy");
    SetWindowMinSize(800, 600);
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

    // fade out the incantation result flashes
    for (auto& r : _state.incantResults)
        r.timer -= dt;
    std::erase_if(_state.incantResults,
                  [](const IncantationResult& r) { return r.timer <= 0.0f; });
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

App::Layout App::computeLayout() const
{
    // map area is the screen minus the right-side info panel
    float availW = (float)GetScreenWidth() - panelWidth() - 2.0f * MARGIN;
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
    if (IsKeyPressed(KEY_F11))
        ToggleBorderlessWindowed();

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
            drawTileResources(_state.map[y][x], rec.x, rec.y, tileSize);
        }
    }

    // active incantations: pulsing golden glow on the ritual tile
    float pulse = 0.5f + 0.5f * (float)std::sin(GetTime() * 6.0);
    for (const auto& inc : _state.incantations) {
        Vector2 c = {
            ox + ((float)inc.x + 0.5f) * tileSize,
            oy + ((float)inc.y + 0.5f) * tileSize
        };
        float rad = tileSize * (0.35f + 0.15f * pulse);
        DrawCircleV(c, rad, { 255, 215, 0, (unsigned char)(50 + 80 * pulse) });
        DrawCircleLines((int)c.x, (int)c.y, rad, GOLD);
    }

    // incantation results: green (success) / red (fail) flash, expanding + fading
    for (const auto& r : _state.incantResults) {
        float a = r.timer / 2.5f; // 1 -> 0
        Vector2 c = {
            ox + ((float)r.x + 0.5f) * tileSize,
            oy + ((float)r.y + 0.5f) * tileSize
        };
        Color col = r.success ? GREEN : RED;
        col.a = (unsigned char)(200 * a);
        DrawCircleV(c, tileSize * (0.4f + 0.5f * (1.0f - a)), col);
    }
    drawLegend(uiScale());

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
    DrawText(TextFormat("Time unit: %d", _state.timeUnit),
             tx, (int)y, hf, LIGHTGRAY); y += hl;
    DrawText(TextFormat("Teams: %d", (int)_state.teams.size()),
             tx, (int)y, hf, LIGHTGRAY); y += 30 * s;

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
