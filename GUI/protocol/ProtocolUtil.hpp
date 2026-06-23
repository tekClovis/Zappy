/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** ProtocolUtil
*/

#pragma once
#include <sstream>
#include <string>

// reads a "#n" token and returns n (or -1 if malformed)
inline int readId(std::istringstream& args)
{
    std::string tok;
    if (!(args >> tok))
        return -1;
    if (!tok.empty() && tok.front() == '#')
        tok.erase(tok.begin());
    try {
        return std::stoi(tok);
    } catch (...) {
        return -1;
    }
}

// reads the rest of the stream as a single trimmed string (for messages)
inline std::string readRest(std::istringstream& args)
{
    std::string rest;
    std::getline(args, rest);
    std::size_t i = rest.find_first_not_of(' ');
    return i == std::string::npos ? "" : rest.substr(i);
}
