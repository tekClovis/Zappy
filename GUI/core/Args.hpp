/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Args
*/

#pragma once
#include <string>

struct Args {
    int port;
    std::string host;
};

Args parseArgs(int ac, char **av);
