#!/bin/bash
mkdir -p src

# Main Cpp file
cat > src/main.cpp << 'EOF'
#include <print>
using std::println;

int main() {
    println("Hello world!");
    return 0;
}
EOF

# .gitignore
cat > .gitignore << 'EOF'
/build
/.cache
/compile_commands.json
EOF

# Makefile
cat > Makefile << 'EOF'
CC = g++
CFLAGS = -Wall -Wextra -O2 -std=c++23 -c
SRC = src
BUILD = build

SRCS := $(wildcard $(SRC)/*.cpp)
OBJS := $(patsubst $(SRC)/%.cpp,$(BUILD)/%.o,$(SRCS))

all: $(BUILD)/main

$(BUILD)/%.o: $(SRC)/%.cpp | $(BUILD)
	bear -- $(CC) $(CFLAGS) $(SRC)/$*.cpp -o $(BUILD)/$*.o

$(BUILD)/main: $(OBJS)
	$(CC) $(OBJS) -o $@

$(BUILD):
	mkdir -p $(BUILD)

run: $(BUILD)/main
	./$(BUILD)/main

clean:
	rm -rf $(BUILD)

.PHONY: all run clean
EOF

# Initialize git repo
git init
