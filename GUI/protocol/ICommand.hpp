/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** ICommand
*/

#pragma once
#include "../game/GameState.hpp"
#include <sstream>

// Command pattern: each GUI-protocol message is handled by a dedicated command
// that applies its effect to the GameState. Adding a message = adding a command
// and registering it (Open/Closed Principle), no central switch to edit.
class ICommand {
public:
    virtual ~ICommand() = default;

    // `args` is the message line with the 3-letter tag already consumed
    virtual void execute(GameState& state, std::istringstream& args) = 0;
};
