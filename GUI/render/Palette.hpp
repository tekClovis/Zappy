/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Palette
*/

#pragma once
#include <raylib.h>

// shared colors/names so both the renderers and the HUD agree
inline const Color TEAM_COLORS[] = {
    RED, BLUE, YELLOW, PURPLE, ORANGE, PINK, SKYBLUE, LIME
};

// index order: 0=food 1=linemate 2=deraumere 3=sibur 4=mendiane 5=phiras 6=thystame
inline const Color RES_COLORS[] = {
    GREEN, RAYWHITE, BROWN, DARKBLUE, VIOLET, MAROON, GOLD
};

inline const char *const RES_NAMES[] = {
    "food", "linemate", "deraumere", "sibur", "mendiane", "phiras", "thystame"
};
