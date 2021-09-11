BIN=pi

CC=/usr/bin/clang
CXX=/usr/bin/clang++
CFLAGS=-std=c89 -Wall -Werror -Wno-comment
CPPFLAGS=$(shell llvm-config --cxxflags --ldflags --libs core bitwriter --system-libs)
INCFLAGS=-Ilib/

OBJ_DIR=./build
SRC_DIR=./src
SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(SRC_FILES))

$(BIN): $(OBJ_FILES)
		$(CXX) $(CPPFLAGS) $(LDFLAGS) -o $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c
		$(CC) $(CFLAGS) $(INCFLAGS) -c -o $@ $<
