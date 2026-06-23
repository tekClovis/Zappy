/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Renderer2D
*/

#include "Renderer2D.hpp"
#include "Palette.hpp"
#include <algorithm>
#include <cmath>
#include <string>

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

static int teamIndex(const GameState& state, const std::string& team)
{
    const auto& teams = state.teams();
    for (int i = 0; i < (int)teams.size(); i++)
        if (teams[i] == team)
            return i;
    return 0;
}

Renderer2D::Layout Renderer2D::computeLayout(const GameState& state,
                                            Rectangle area) const
{
    float tileSize = std::min(area.width / (float)state.width(),
                              area.height / (float)state.height());
    return {
        tileSize,
        area.x + (area.width - tileSize * (float)state.width()) / 2.0f,
        area.y + (area.height - tileSize * (float)state.height()) / 2.0f
    };
}

void Renderer2D::update(const GameState&, float)
{
    // no camera state (the 2D view always fits the map to the area)
}

void Renderer2D::draw(const GameState& state, Rectangle area, int selected)
{
    Layout l = computeLayout(state, area);
    float tileSize = l.tileSize;
    float ox = l.ox;
    float oy = l.oy;
    float scale = (float)GetScreenHeight() / 720.0f;

    // tiles + resources
    for (int y = 0; y < state.height(); y++) {
        for (int x = 0; x < state.width(); x++) {
            Rectangle rec = {
                ox + (float)x * tileSize, oy + (float)y * tileSize,
                tileSize - 1.0f, tileSize - 1.0f
            };
            DrawRectangleRec(rec, { 34, 85, 34, 255 });
            drawTileResources(state.tile(x, y), rec.x, rec.y, tileSize);
        }
    }

    // active incantations: pulsing golden glow on the ritual tile
    float pulse = 0.5f + 0.5f * (float)std::sin(GetTime() * 6.0);
    for (const auto& inc : state.incantations()) {
        Vector2 c = {
            ox + ((float)inc.x + 0.5f) * tileSize,
            oy + ((float)inc.y + 0.5f) * tileSize
        };
        float rad = tileSize * (0.35f + 0.15f * pulse);
        DrawCircleV(c, rad, { 255, 215, 0, (unsigned char)(50 + 80 * pulse) });
        DrawCircleLines((int)c.x, (int)c.y, rad, GOLD);
    }

    // incantation results: green (success) / red (fail) flash, expanding + fading
    for (const auto& r : state.incantResults()) {
        float a = r.timer / 2.5f;
        Vector2 c = {
            ox + ((float)r.x + 0.5f) * tileSize,
            oy + ((float)r.y + 0.5f) * tileSize
        };
        Color col = r.success ? GREEN : RED;
        col.a = (unsigned char)(200 * a);
        DrawCircleV(c, tileSize * (0.4f + 0.5f * (1.0f - a)), col);
    }
    drawLegend(scale);

    // eggs (pop-in scale during the first 0.4s after spawn)
    for (const auto& [id, egg] : state.eggs()) {
        float ex = ox + ((float)egg.x + 0.5f) * tileSize;
        float ey = oy + ((float)egg.y + 0.5f) * tileSize;
        float pop = std::min(1.0f, egg.age / 0.4f);
        DrawCircle((int)ex, (int)ey, tileSize * 0.15f * pop, WHITE);
        DrawCircleLines((int)ex, (int)ey, tileSize * 0.15f * pop, { 200, 200, 200, 255 });
    }

    // egg hatch (green) / death (gray) bursts
    for (const auto& f : state.eggFx()) {
        float a = f.timer / 0.6f;
        Vector2 c = {
            ox + ((float)f.x + 0.5f) * tileSize,
            oy + ((float)f.y + 0.5f) * tileSize
        };
        Color col = f.hatched ? GREEN : GRAY;
        col.a = (unsigned char)(220 * a);
        DrawCircleLines((int)c.x, (int)c.y, tileSize * (0.15f + 0.4f * (1.0f - a)), col);
    }

    // players (triangle pointing toward the facing direction)
    for (const auto& [id, p] : state.players()) {
        Vector2 center = {
            ox + (p.renderX + 0.5f) * tileSize,
            oy + (p.renderY + 0.5f) * tileSize
        };
        Color c = TEAM_COLORS[teamIndex(state, p.team) % 8];
        float radius = tileSize * 0.30f;
        float rot = (float)(p.orientation - 2) * 90.0f;

        if (id == selected)
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

    // broadcasts: expanding blue sound wave + message bubble from the emitter
    for (const auto& b : state.broadcasts()) {
        auto it = state.players().find(b.playerId);
        if (it == state.players().end())
            continue;
        Vector2 c = {
            ox + (it->second.renderX + 0.5f) * tileSize,
            oy + (it->second.renderY + 0.5f) * tileSize
        };
        float a = b.timer / 3.0f;
        float grow = 1.0f - a;
        DrawCircleLines((int)c.x, (int)c.y, tileSize * (0.5f + 2.5f * grow),
                        { 0, 170, 255, (unsigned char)(220 * a) });
        if (!b.text.empty()) {
            int fs = (int)(14 * scale);
            int w = MeasureText(b.text.c_str(), fs);
            DrawText(b.text.c_str(), (int)(c.x - (float)w / 2.0f),
                     (int)(c.y - tileSize * 0.6f - (float)fs), fs, SKYBLUE);
        }
    }

    // ejections: quick orange burst on the player's tile
    for (const auto& e : state.ejects()) {
        auto it = state.players().find(e.playerId);
        if (it == state.players().end())
            continue;
        Vector2 c = {
            ox + (it->second.renderX + 0.5f) * tileSize,
            oy + (it->second.renderY + 0.5f) * tileSize
        };
        float a = e.timer;
        DrawCircleV(c, tileSize * (0.3f + 0.6f * (1.0f - a)),
                    { 255, 140, 0, (unsigned char)(180 * a) });
    }
}

int Renderer2D::pickPlayer(const GameState& state, Rectangle area, Vector2 mouse)
{
    Layout l = computeLayout(state, area);
    int tx = (int)((mouse.x - l.ox) / l.tileSize);
    int ty = (int)((mouse.y - l.oy) / l.tileSize);

    if (tx < 0 || tx >= state.width() || ty < 0 || ty >= state.height())
        return -1;
    for (const auto& [id, p] : state.players())
        if (p.x == tx && p.y == ty)
            return id;
    return -1;
}
