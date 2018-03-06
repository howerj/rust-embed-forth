
TARGET=embed

ifeq ($(OS),Windows_NT)
EXE=.exe
RM=del
DF=
else
EXE=
RM=rm -vf
DF=./
endif

.PHONY: all clean build run

all: run

build:
	cargo build

run: 
	cargo run

clean:
	cargo clean
