/*
** EPITECH PROJECT, 2026
** Zappy
** File description:
** Commands
*/

#pragma once
#include "ICommand.hpp"

// Each concrete command handles one GUI-protocol message. They are registered
// by tag in the CommandDispatcher.

#define ZAPPY_COMMAND(Name)                                              \
    class Name : public ICommand {                                       \
    public:                                                              \
        void execute(GameState& state, std::istringstream& args) override; \
    }

// --- map / world ---
ZAPPY_COMMAND(MapSizeCommand);      // msz
ZAPPY_COMMAND(TileContentCommand);  // bct

// --- teams / server ---
ZAPPY_COMMAND(TeamNameCommand);     // tna
ZAPPY_COMMAND(TimeUnitGetCommand);  // sgt
ZAPPY_COMMAND(TimeUnitSetCommand);  // sst
ZAPPY_COMMAND(EndGameCommand);      // seg
ZAPPY_COMMAND(ServerMessageCommand);// smg
ZAPPY_COMMAND(UnknownCommand);      // suc
ZAPPY_COMMAND(BadParameterCommand); // sbp

// --- players ---
ZAPPY_COMMAND(NewPlayerCommand);        // pnw
ZAPPY_COMMAND(PlayerPositionCommand);   // ppo
ZAPPY_COMMAND(PlayerLevelCommand);      // plv
ZAPPY_COMMAND(PlayerInventoryCommand);  // pin
ZAPPY_COMMAND(PlayerDeathCommand);      // pdi
ZAPPY_COMMAND(IncantationStartCommand); // pic
ZAPPY_COMMAND(IncantationEndCommand);   // pie
ZAPPY_COMMAND(PlayerDropCommand);       // pdr
ZAPPY_COMMAND(PlayerTakeCommand);       // pgt
ZAPPY_COMMAND(BroadcastCommand);        // pbc
ZAPPY_COMMAND(EjectCommand);            // pex

// --- eggs ---
ZAPPY_COMMAND(EggLaidCommand);   // enw
ZAPPY_COMMAND(EggHatchCommand);  // ebo
ZAPPY_COMMAND(EggDeathCommand);  // edi

#undef ZAPPY_COMMAND
