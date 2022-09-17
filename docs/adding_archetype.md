# Adding a new archetype

## Adding a new icon

- Find the path of the icon in the icons zip
- Add tag/path pair to `icons/build.rs`

## Adding entity archetype

- Add row to `game-config.xlsx` `stuff-desciptor` sheet
- Fix compile errors
- Add row to `enemy-chances` or `item-chances` sheet in `game-config.xlsx` to spawn the new archetype
- Test

## Adding components

- Add components to `components.rs`
- Add systems to `systems.rs`
- (if configurable) Add column to game-config.xlsx
- (if configurable) Add columns to `StuffDescription` in `core/build.rs`
