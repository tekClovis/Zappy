/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Dispatcher
*/

#include "Dispatcher.hpp"
#include "Commands.hpp"

CommandDispatcher::CommandDispatcher(GameState& state) : _state(state)
{
    registerCommand("msz", std::make_unique<MapSizeCommand>());
    registerCommand("bct", std::make_unique<TileContentCommand>());

    registerCommand("tna", std::make_unique<TeamNameCommand>());
    registerCommand("sgt", std::make_unique<TimeUnitGetCommand>());
    registerCommand("sst", std::make_unique<TimeUnitSetCommand>());
    registerCommand("seg", std::make_unique<EndGameCommand>());
    registerCommand("smg", std::make_unique<ServerMessageCommand>());
    registerCommand("suc", std::make_unique<UnknownCommand>());
    registerCommand("sbp", std::make_unique<BadParameterCommand>());

    registerCommand("pnw", std::make_unique<NewPlayerCommand>());
    registerCommand("ppo", std::make_unique<PlayerPositionCommand>());
    registerCommand("plv", std::make_unique<PlayerLevelCommand>());
    registerCommand("pin", std::make_unique<PlayerInventoryCommand>());
    registerCommand("pdi", std::make_unique<PlayerDeathCommand>());
    registerCommand("pic", std::make_unique<IncantationStartCommand>());
    registerCommand("pie", std::make_unique<IncantationEndCommand>());
    registerCommand("pdr", std::make_unique<PlayerDropCommand>());
    registerCommand("pgt", std::make_unique<PlayerTakeCommand>());
    registerCommand("pbc", std::make_unique<BroadcastCommand>());
    registerCommand("pex", std::make_unique<EjectCommand>());

    registerCommand("enw", std::make_unique<EggLaidCommand>());
    registerCommand("ebo", std::make_unique<EggHatchCommand>());
    registerCommand("edi", std::make_unique<EggDeathCommand>());
}

void CommandDispatcher::registerCommand(const std::string& tag,
                                        std::unique_ptr<ICommand> cmd)
{
    _commands[tag] = std::move(cmd);
}

void CommandDispatcher::dispatch(const std::string& line)
{
    if (line.size() < 3)
        return;
    auto it = _commands.find(line.substr(0, 3));
    if (it == _commands.end())
        return;
    std::istringstream args(line.substr(3));
    it->second->execute(_state, args);
}
