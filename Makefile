BIN=pi

CC=/usr/bin/clang
CXX=/usr/bin/clang++
CFLAGS=-std=c89 -Wall -Werror -Wno-comment
CPPFLAGS=$(shell llvm-config --cxxflags --ldflags --libs core bitwriter --system-libs)
# LDFLAGS=-lasan
INCFLAGS=-Ilib/

OBJ_DIR=./build
SRC_DIR=./src
# SRC_FILES=$(SRC_DIR)/pi.c $(SRC_DIR)/token.c $(SRC_DIR)/scanner.c $(SRC_DIR)/debug.c $(SRC_DIR)/parser.c $(SRC_DIR)/ir.c
SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(SRC_FILES))

$(BIN): $(OBJ_FILES)
		$(CXX) $(CPPFLAGS) $(LDFLAGS) -o $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c
		$(CC) $(CFLAGS) $(INCFLAGS) -c -o $@ $<

# %.o: $(SRC_DIR)/%.c
# 		$(CC) $(CFLAGS) $(shell llvm-config --cflags) -o $@ $<

# $(BIN): *.o
# 		$(CXX) $(shell llvm-config --cxxflags --ldflags --libs core bitwriter --system-libs) $< -o $(BIN)

# all:
#		$(CC) $(CFLAGS) -o $(BIN) $(SRC_FILES)

# all:
#		$(CC) $(CFLAGS) $(shell llvm-config --cflags) -c $(SRC_FILES)
#		$(CXX) $(shell llvm-config --cxxflags --ldflags --libs core bitwriter --system-libs) pi.o -o $(BIN)