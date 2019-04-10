PACKAGE := crayon-cli
PWD := $(shell pwd)

ifeq ($(OS),Windows_NT)
    SYSTEM := Windows
else
    SYSTEM := $(shell uname -s)
endif

ifeq ($(SYSTEM),Windows)
	ASSIMP := utilities\assimp\bin_x64
	CRUNCH := utilities\crunch\bin_mingw\crunch
	PVRTEX := utilities\PVRTexTool\CLI\Windows_x86_64\PVRTexToolCLI.exe
$(shell mkdir "C:\Users\admin\crayon-cli")
	PWD := $(shell chd)
	SYSMBOL_PATH := "C:\Users\admin"
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
	ifeq ($(SYSTEM),Windows)
	$(shell mkdir $(DESTDIR)\$(PACKAGE))
	$(shell mkdir $(DESTDIR)\$(PACKAGE)\utilities)
	$(shell Xcopy /Y $(ASSIMP) $(DESTDIR)\$(PACKAGE)\utilities )
	$(shell Xcopy /Y $(CRUNCH) $(DESTDIR)\$(PACKAGE)\utilities\CRUNCH )
	$(shell Xcopy /Y $(PVRTEX) $(DESTDIR)\$(PACKAGE)\utilities\PVRTexToolCLI )
	$(shell Xcopy /Y target\release\crayon-cli $(DESTDIR)\$(PACKAGE)\crayon-cli )
	$(shell mklink $(PWD$)\$(DESTDIR)\$(PACKAGE)\crayon-cli $(SYSMBOL_PATH))
	else
		mkdir -p $(DESTDIR)/$(PACKAGE)
		mkdir -p $(DESTDIR)/$(PACKAGE)/utilities
		cp $(ASSIMP) $(DESTDIR)/$(PACKAGE)/utilities/assimp
		cp -r $(CRUNCH) $(DESTDIR)/$(PACKAGE)/utilities/crunch
		cp $(PVRTEX) $(DESTDIR)/$(PACKAGE)/utilities/PVRTexToolCLI
		cp target/release/crayon-cli $(DESTDIR)/$(PACKAGE)/crayon-cli
		sudo ln -sf $(PWD$)/$(DESTDIR)/$(PACKAGE)/crayon-cli $(SYSMBOL_PATH)/crayon-cli
	endif
build:
	cargo --color always build --release

uninstall:
	rm -rf $(DESTDIR)/$(PACKAGE)
	rm -f $(SYSMBOL_PATH)/crayon-cli
