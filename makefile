PACKAGE := crayon-cli
PWD := $(shell pwd)

ifeq ($(OS),Windows_NT)
    SYSTEM := Windows
else
    SYSTEM := $(shell uname -s)
endif

ifeq ($(SYSTEM),Windows)
	$(error Windows supports is not implemented yet.)
endif

ifeq ($(SYSTEM),Darwin)
	ASSIMP := utilities/assimp/bin_osx/assimp
	CRUNCH := utilities/crunch/bin_osx/crunch
	PVRTEX := utilities/PVRTexTool/CLI/OSX_x86/PVRTexToolCLI
	SYSMBOL_PATH := /usr/local/bin
endif

ifeq ($(SYSTEM),Linux)
	ASSIMP := utilities/assimp/bin_linux/assimp
	CRUNCH := utilities/crunch/bin_linux/crunch
	PVRTEX := utilities/PVRTexTool/CLI/linux/PVRTexToolCLI
	SYSMBOL_PATH := /usr/local/bin
endif

ifndef DESTDIR
    $(warning DESTDIR is undefined)
	DESTDIR := target
endif

all: install

install: build
	mkdir -p $(DESTDIR)/$(PACKAGE)
	mkdir -p $(DESTDIR)/$(PACKAGE)/utilities

	cp $(ASSIMP) $(DESTDIR)/$(PACKAGE)/utilities/assimp
	cp -r $(CRUNCH) $(DESTDIR)/$(PACKAGE)/utilities/crunch
	cp $(PVRTEX) $(DESTDIR)/$(PACKAGE)/utilities/PVRTexToolCLI
	cp target/release/crayon-cli $(DESTDIR)/$(PACKAGE)/crayon-cli

	sudo ln -sf $(PWD$)/$(DESTDIR)/$(PACKAGE)/crayon-cli $(SYSMBOL_PATH)/crayon-cli

build:
	cargo --color always build --release

uninstall:
	rm -rf $(DESTDIR)/$(PACKAGE)
	rm -f $(SYSMBOL_PATH)/crayon-cli
