/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Dispatcher
*/

#pragma once
#include "ICommand.hpp"
#include <memory>
#include <string>
#include <unordered_map>

// Maps each 3-letter protocol tag to its command and dispatches incoming lines.
class CommandDispatcher {
public:
    explicit CommandDispatcher(GameState& state);
    void dispatch(const std::string& line);

private:
    void registerCommand(const std::string& tag, std::unique_ptr<ICommand> cmd);

    GameState& _state;
    std::unordered_map<std::string, std::unique_ptr<ICommand>> _commands;
};
