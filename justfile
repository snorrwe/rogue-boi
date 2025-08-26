dev:
    npm run dev

update:
    nix flake update
    cargo update --breaking --recursive -Z unstable-options
    cargo update
    npm update --force

clean:
    rm -rf rogue-boi-core/pkg
    rm -rf ui/public/icons
    rm -rf ui/dist
    cargo clean

format:
    npm run format
    cargo fmt

# initialize the project
init:
    git lfs pull
    npm i
