#!/bin/bash
echo '#include <iostream>

int main() {
    std::cout << "Hello world!" << std::endl;
    return 0;
}' > main.cpp
git init
