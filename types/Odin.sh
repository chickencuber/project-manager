#!/bin/bash
echo 'package main

import "core:fmt"

main :: proc() {
	fmt.println("Hello World!")
}' > main.odin
git init
