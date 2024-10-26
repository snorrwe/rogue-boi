<script>
  import { coreStore, icons as iconsSvg } from "@rogueBoi/store.js";

  let core;
  let dungeon;
  let tiles = $state([]);
  let icons = $state();
  let level = $state(1);

  iconsSvg.subscribe((i) => {
    icons = {};
    for (const [k, v] of Object.entries(i)) {
      icons[k] = `data:image/svg+xml;base64,${btoa(v)}`;
    }
  });

  let dims = $state([64, 64]); // FIXME: query from core, or set on dungeon generation
  let desiredDims = $state([64, 64]);

  coreStore.subscribe((c) => {
    core = c;
    regenerate({ dims, level });
  });
  // Debug ui for dungeon generation
  function regenerate({ dims, level }) {
    dungeon = core.generate_dungeon({ level, dims: { x: dims[0], y: dims[1] } });
    tiles.length = 0;
    for (let y = 0; y < dims[1]; ++y) {
      for (let x = 0; x < dims[0]; ++x) {
        tiles.push({ x, y, icon: dungeon.get(`${x};${y}`) });
      }
    }
  }
</script>

<div class="content">
  <div>Debug tool to visualize the map generator's behaviour</div>
  <div>
    <form
      class="input"
      onsubmit={(e) => {
        dims = desiredDims;
        regenerate({ dims, level });
        e.preventDefault();
      }}
    >
      <div>
        <label for="level">Level</label>
        <input type="number" name="level" placeholder="level" min="1" bind:value={level} />
      </div>
      <div>
        <label for="x">Width</label>
        <input type="number" name="x" placeholder="x" min="50" bind:value={desiredDims[0]} />
        <label for="y">Height</label>
        <input type="number" name="y" placeholder="y" min="50" bind:value={desiredDims[1]} />
      </div>
      <div>
        <button type="submit">Regen</button>
      </div>
    </form>
    <div class="grid" style="--cols:{dims[0]}">
      {#each tiles as tile}
        {#if tile.icon}
          <img
            src={icons[tile.icon]}
            alt={tile.icon}
            title={`(${tile.x}, ${tile.y}): ${tile.icon}`}
          />
        {:else}
          <div></div>
        {/if}
      {/each}
    </div>
  </div>
</div>

<style>
  .content {
    max-width: 960px;
    margin: auto;
  }

  .input {
    max-height: 20%;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), 1fr);
    max-height: 80%;
  }
</style>
