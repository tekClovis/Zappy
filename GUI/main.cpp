/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** main
*/

#include "core/App.hpp"
#include "core/Args.hpp"
#include <iostream>

int main(int ac, char **av)
{
    Args args = parseArgs(ac, av);
    try {
        App app(args);
        app.run();
    } catch (const std::exception& e) {
        std::cerr << e.what() << "\n";
        return 84;
    }
    return 0;
}
