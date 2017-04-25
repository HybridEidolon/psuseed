# psuseed

[![Build status](https://ci.appveyor.com/api/projects/status/6jr8et2g5ik4rf3k/branch/master?svg=true)](https://ci.appveyor.com/project/HybridEidolon/psuseed/branch/master)

A small dinput8 wrapper to add a few no-nonsense tweaks to Phantasy Star
Universe, made for the Clementine private server.

- [Releases/Downloads](https://github.com/HybridEidolon/psuseed/releases)
- [Latest CI builds](https://ci.appveyor.com/project/HybridEidolon/psuseed/build/artifacts)

## Usage

Extract the plugin to the root of your PSU: AOTI installation, edit the
psuseed.toml config with a text editor, and run the game.

In order for memory patches to work, you must be using an uncompressed
PSU exe. Without it, only the borderless window patch will work.

Clementine executables are compressed with UPX; if you want to decompress
it yourself, download UPX from [here](https://upx.github.io/) and run the
following in a command line in your PSU directory:

    upx.exe -d -o PSUC_Uncompressed.exe PSUC.exe

## Features

- Configurable via psuseed.toml
- Arbitrary resolution support. 4K!
- Disable minimap for non-1280x720 (otherwise it bugs)
- Override server hostname and ports (for you sneaky server devs)

## Development

Use nightly i686 msvc rust (not gnu) with VS 2015 build tools.

    $ cargo build --release
    $ copy target/release/dinput8.dll path/to/psudir
