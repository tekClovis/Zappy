/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Network
*/

#include "Network.hpp"
#include <fcntl.h>
#include <netdb.h>
#include <sys/socket.h>
#include <unistd.h>
#include <cerrno>
#include <stdexcept>

Network::~Network()
{
    if (_fd >= 0) close(_fd);
}

void Network::connect(const std::string& host, int port)
{
    addrinfo hints{}, *res;
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    if (getaddrinfo(host.c_str(), std::to_string(port).c_str(), &hints, &res) != 0)
        throw std::runtime_error("Cannot resolve host: " + host);

    _fd = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (_fd < 0) {
        freeaddrinfo(res);
        throw std::runtime_error("socket() failed");
    }
    if (::connect(_fd, res->ai_addr, res->ai_addrlen) < 0) {
        freeaddrinfo(res);
        close(_fd);
        _fd = -1;
        throw std::runtime_error("Cannot connect to " + host + ":" + std::to_string(port));
    }
    freeaddrinfo(res);

    int flags = fcntl(_fd, F_GETFL, 0);
    fcntl(_fd, F_SETFL, flags | O_NONBLOCK);
}

void Network::send(const std::string& msg)
{
    if (_fd < 0)
        return;
    ::send(_fd, msg.c_str(), msg.size(), 0);
}

void Network::update()
{
    if (_fd < 0)
        return;

    char buf[4096];
    ssize_t n;
    while ((n = recv(_fd, buf, sizeof(buf), 0)) > 0)
        _buf.append(buf, n);

    // n == 0: server closed the connection; n < 0 with a real error: drop it
    if (n == 0 || (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK)) {
        close(_fd);
        _fd = -1;
    }
}

std::optional<std::string> Network::nextLine()
{
    auto pos = _buf.find('\n');
    if (pos == std::string::npos)
        return std::nullopt;
    std::string line = _buf.substr(0, pos);
    _buf.erase(0, pos + 1);
    if (!line.empty() && line.back() == '\r')
        line.pop_back();
    return line;
}
