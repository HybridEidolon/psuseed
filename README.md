# psuseed

[![Build status](https://ci.appveyor.com/api/projects/status/6jr8et2g5ik4rf3k/branch/master?svg=true)](https://ci.appveyor.com/project/HybridEidolon/psuseed/branch/master)

A small dinput8 wrapper to add a few no-nonsense tweaks to Phantasy Star
Universe, made for the Clementine private server.

[Releases/Downloads](https://github.com/HybridEidolon/psuseed/releases)

## Usage

Extract the plugin to the root of your PSU: AOTI installation, edit the
psuseed.toml config with a text editor, and run the game.

## Development

Use nightly i686 msvc rust (not gnu) with VS 2015 build tools.

    $ cargo build --release
    $ copy target/release/dinput8.dll path/to/psudir
