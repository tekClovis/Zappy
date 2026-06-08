##
## EPITECH PROJECT, 2024
## Makefile
## File description:
## Makefile
##

NAME = zappy

SERV_BIN = $(NAME)_server
SERV_DIR = server
SERV_MANIFEST = $(SERV_DIR)/Cargo.toml
SERV_TARGET_DIR = $(SERV_DIR)/target
SERV_TARGET = $(SERV_TARGET_DIR)/debug/$(SERV_BIN)

AI_BIN = $(NAME)_ai
SRC_AI = AI/main.cpp \

GUI_BIN = $(NAME)_gui
SRC_GUI = GUI/main.cpp \

OBJ_AI = $(SRC_AI:.cpp=.o)
OBJ_GUI = $(SRC_GUI:.cpp=.o)

WARNINGS = -Wextra -Wall -Werror -std=c++20

AI_INCLUDES = -I./AI
GUI_INCLUDES = -I./GUI
COMMON_INCLUDES = -I./include

MAKEFLAGS = --colors=auto

%.o: %.cpp
	g++ -fPIC -c $< -o $@ $(CFLAGS) $(WARNINGS)

all:
	$(MAKE) $(MAKEFLAGS) zappy_server
	$(MAKE) $(MAKEFLAGS) zappy_ai
	$(MAKE) $(MAKEFLAGS) zappy_gui

fast: MAKEFLAGS += -j
fast: all

zappy_server:
	cargo build --manifest-path $(SERV_MANIFEST)
	cp -f $(SERV_TARGET) $(SERV_BIN)

zappy_ai: $(OBJ_AI)
	clang++ $(CFLAGS) $(WARNINGS) $(OBJ_AI) -o $(AI_BIN)

zappy_gui: $(OBJ_GUI)
	clang++ $(CFLAGS) $(WARNINGS) $(OBJ_GUI) -o $(GUI_BIN)



debug:
	CFLAGS += -g3
	all

clean:
	rm -f $(OBJ_AI) $(OBJ_GUI)
	cargo clean --manifest-path $(SERV_MANIFEST)

fclean: clean
	rm -f $(SERV_BIN) $(AI_BIN) $(GUI_BIN)

re:	fclean all

.PHONY: all fast zappy_server zappy_ai zappy_gui debug clean fclean re
