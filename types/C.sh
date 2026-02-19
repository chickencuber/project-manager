#!/bin/bash

mkdir -p src

# Main C file
cat > src/main.c << 'EOF'
#include <stdio.h>

int main() {
    printf("Hello World!\n");
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
CC = gcc
CFLAGS = -Wall -Wextra -O2 -c
SRC = src
BUILD = build

SRCS := $(wildcard $(SRC)/*.c)
OBJS := $(patsubst $(SRC)/%.c, $(BUILD)/%.o,$(SRCS))

all: $(BUILD)/main

$(BUILD)/%.o: $(SRC)/%.c | $(BUILD)
	bear -- $(CC) $(CFLAGS) $(SRC)/$*.c -o $(BUILD)/$*.o

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
