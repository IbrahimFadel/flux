BIN=pi
BUILD_DIR=./build

CC=/usr/bin/clang
CXX=/usr/bin/clang++
CFLAGS=-std=c99 -Wall -Werror -Wno-comment -g
CPPFLAGS=$(shell llvm-config --cxxflags --ldflags --libs core --system-libs)
INCFLAGS=-Ilib/
LDFLAGS=
SUBDIRS=./lib/sds

SRC_DIR=./src
SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(SRC_FILES))

all: $(SUBDIRS) $(BUILD_DIR)/$(BIN)

$(BUILD_DIR)/$(BIN): $(OBJ_FILES) ./lib/sds/sds.o
		$(CXX) $(CPPFLAGS) $(LDFLAGS) -o $@ $^

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
		$(CC) $(CFLAGS) $(INCFLAGS) -c -o $@ $<

$(SUBDIRS):
		$(MAKE) -C $@

.PHONY: all $(SUBDIRS)