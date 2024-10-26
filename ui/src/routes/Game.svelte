<script>
  import { onMount } from "svelte";
  import {
    canvasStore,
    coreStore,
    coreOutput,
    inventory,
    equipment,
    selected
  } from "@rogueBoi/store.js";
  import Grid from "@rogueBoi/game/Grid.svelte";
  import Highlight from "@rogueBoi/game/Highlight.svelte";
  import Log from "@rogueBoi/game/Log.svelte";
  import Player from "@rogueBoi/game/Player.svelte";

  let core = $state();
  coreStore.subscribe((c) => (core = c));

  let last = performance.now();

  canvasStore.subscribe((canvas) => core.setCanvas(canvas));

  function onKey(event) {
    core.pushEvent({
      ty: "KeyDown",
      key: event.key
    });
  }

  function gameLoop(now) {
    if (core) {
      core.tick(now - last);

      const output = core.getOutput();
      coreOutput.set(output);
      inventory.set(core.getInventory());
      equipment.set(core.getEquipment());
      last = now;
      if (output && output.selected) {
        // TODO: include the entity information in selected field
        selected.set(core.fetchEntity(output.selected));
      }
    }
    requestAnimationFrame(gameLoop);
  }
  requestAnimationFrame(gameLoop);

  onMount(() => {
    let autoSaveHandle = setInterval(() => {
      if ($coreStore) {
        let pl = $coreStore.save();
        localStorage.setItem("save", pl);
      }
    }, 30000);
    return () => clearInterval(autoSaveHandle);
  });
</script>

<svelte:window onkeydown={onKey} />

<main>
  {#if core != null}
    <div class="content">
      <div class="log">
        <h2>Log</h2>
        <Log log={$coreOutput.log} />
      </div>
      <div>
        <div>Dungeon floor: {$coreOutput.dungeonLevel}</div>
        <Grid />
      </div>
      <div class="game-ui">
        {#if $selected != null}
          <Highlight targetingMode={$coreOutput.targeting} selected={$selected} {core} />
        {/if}
        <div>
          <Player />
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  main {
    padding: 2em;
    margin: 0 auto;
    color: white;
    min-width: 0px;
    box-sizing: border-box;
    height: 90vh;
  }

  .content {
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    column-gap: 2rem;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .log {
    max-height: 100%;
    overflow: auto;
  }

  .game-ui {
    color: white;
    text-align: left;
  }
</style>
