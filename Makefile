BIN=pi
BUILD_DIR=./build

CC=/usr/bin/clang
CXX=/usr/bin/clang++
CFLAGS=-std=c89 -Wall -Werror -Wno-comment
CPPFLAGS=$(shell llvm-config --cxxflags --ldflags --libs core bitwriter --system-libs)
INCFLAGS=-Ilib/

SRC_DIR=./src
SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(SRC_FILES))

$(BUILD_DIR)/$(BIN): $(OBJ_FILES)
		$(CXX) $(CPPFLAGS) $(LDFLAGS) -o $@ $^

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
		$(CC) $(CFLAGS) $(INCFLAGS) -c -o $@ $<
