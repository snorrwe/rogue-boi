dev:
    npm run dev

update:
    cargo update
    npm update
    nix flake update

clean:
    rm -rf rogue-boi-core/pkg
    rm -rf ui/public/icons
    rm -rf ui/dist
    cargo clean

format:
    npm run format
    cargo fmt
