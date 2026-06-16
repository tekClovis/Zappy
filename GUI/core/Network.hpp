/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Network
*/

#pragma once
#include <optional>
#include <string>

class Network {
public:
    Network() = default;
    ~Network();

    void connect(const std::string& host, int port);
    void send(const std::string& msg);
    void update();
    std::optional<std::string> nextLine();
    bool isConnected() const { return _fd >= 0; }

private:
    int _fd = -1;
    std::string _buf;
};
