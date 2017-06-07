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
- Dynamic filename hash recovery. See below for instructions.

### Configuring

Edit the `psuseed.toml` file and change the options as you desire.

### Using unhashed DATA file names

This is an experimental feature that is _mostly_ stable. It will hook
the file name hashing function and instead return the actual name that
the client is searching for. If you set the `hashed_names_path`, the
plugin will first check that the real name's file exists in your DATA
directory. If the file doesn't exist, it will copy the hashed file
into your DATA directory under its new name and resume.

The file copying is a blocking procedure and will incur significant
load times and lag at first, but will progressively get better as
you continue to play. **In particular, the initial startup will be
very very slow, as the sound files are >400mb a piece.** Please make
sure to let it complete the copying procedure and **exit the game
naturally whenever possible.**

You can also find a list of files online and pre-copy all of them
to the original names yourself to relieve a lot of the initial build
up time.

## Development

Use nightly i686 msvc rust (not gnu) with VS 2015 build tools.

    $ cargo build --release
    $ copy target/release/dinput8.dll path/to/psudir
