prog :=mlat-client-rust

DEBUG ?= no
COMPRESS ?= no

ifeq ($(DEBUG), yes)
	release :=
	target :=debug
else
	release :=--release
	target :=release
endif

build:
	cargo build $(release)

install:
ifeq ($(DEBUG), yes)
	cp target/$(target)/$(prog) /usr/local/bin/$(prog)-debug
	chmod 755 /usr/local/bin/$(prog)-debug
ifeq ($(COMPRESS), yes)
	upx --best --lzma /usr/local/bin/$(prog)-debug
endif
else
	cp target/$(target)/$(prog) /usr/local/bin/$(prog)
	chmod 755 /usr/bin/$(prog)
ifeq ($(COMPRESS), yes)
	upx --best --lzma /usr/local/bin/$(prog)
endif
endif

all: build install

uninstall:
ifeq ($(DEBUG), yes)
	rm /usr/local/bin/$(prog)-debug
else
	rm /usr/local/bin/$(prog)
endif

help:
	@echo "usage: make $(prog) [debug=1]"
