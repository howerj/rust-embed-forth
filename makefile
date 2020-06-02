
TARGET=embed
IMAGE=eforth.blk
NEW=new.blk

ifeq ($(OS),Windows_NT)
EXE=.exe
RM=del
DF=
else
EXE=
RM=rm -vf
DF=./
endif

.PHONY: all clean build run release test

all: test doc build

build:
	cargo build

release:
	cargo build --release

run: 
	cargo run ${IMAGE} ${NEW}

test:
	cargo test

doc:
	cargo doc

clean:
	cargo clean
