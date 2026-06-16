/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Args
*/

#include "Args.hpp"
#include <cstdlib>
#include <iostream>

static void usage(const char *prog)
{
    std::cerr << "USAGE: " << prog << " -p port -h machine\n"
              << "  -p port     port number\n"
              << "  -h machine  hostname of the server\n";
    exit(84);
}

Args parseArgs(int ac, char **av)
{
    Args args{ -1, "localhost" };

    for (int i = 1; i < ac; i++) {
        std::string opt = av[i];
        if ((opt == "-p" || opt == "-h") && i + 1 >= ac)
            usage(av[0]);
        if (opt == "-p")
            args.port = std::atoi(av[++i]);
        else if (opt == "-h")
            args.host = av[++i];
        else if (opt == "--help")
            usage(av[0]);
        else
            usage(av[0]);
    }
    if (args.port == -1)
        usage(av[0]);
    return args;
}
