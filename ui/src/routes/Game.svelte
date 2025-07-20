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
  import Help from "@rogueBoi/game/Help.svelte";

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

<main class="text-white min-w-0 box-border md:h-[90vh] mx-auto my-0 p-[2em] overflow-auto">
  {#if core != null}
    <div
      class="grid md:grid-cols-[1fr_2fr_1fr] sm:grid-cols-[1fr_2fr] gap-x-8 w-full h-full object-contain"
    >
      <div class="text-white text-left">
        {#if $selected != null}
          <Highlight targetingMode={$coreOutput.targeting} selected={$selected} {core} />
        {/if}
        <div>
          <Player />
        </div>
        <div>
          <Help />
        </div>
      </div>

      <div class="min-w-[426px] min-h-[426px]">
        <div>Dungeon floor: {$coreOutput.dungeonLevel}</div>
        <Grid />
      </div>

      <div class="max-h-full overflow-auto">
        <h2>Log</h2>
        <Log log={$coreOutput.log} />
      </div>
    </div>
  {/if}
</main>
