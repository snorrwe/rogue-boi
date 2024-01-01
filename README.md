# Rogue Boi

A classic rogue-like in the browser. Started by loosely following http://rogueliketutorials.com/tutorials/tcod/v2/ .

The game is available on [itch.io](https://snorrwe.itch.io/rogue-boi).

I use this project as both a recreational coding excercise and to try out new techniques and libraries.
For this reason the codebase is at times ridiculously over-engineered for what it does.

## Start local

### Install dependencies

You can either install dependencies yourself or use [Nix](https://nixos.org/manual/nix/stable/installation/installation.html)

Manual installation:

- [rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [nodejs](https://nodejs.org/en/)
- [just](https://github.com/casey/just)
- Run `npm i`

Via Nix:

- Install [nix](https://nixos.org/download)
- Install [direnv](https://direnv.net/)
- Run `direnv allow`

Alternatively: install nix, then run `nix develop` to get a shell with the required tools installed.

### Run

```sh
just dev
```

### Icons

Icons are from [game-icons.net](https://game-icons.net/)

### Generating the CHANGELOG

```sh
git cliff
```
