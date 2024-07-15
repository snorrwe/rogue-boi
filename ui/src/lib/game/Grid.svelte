<script>
  import { onDestroy, onMount } from "svelte";
  import { canvasStore } from "@rogueBoi/store.js";
  import { writable } from "svelte/store";

  let box;
  let canvas;
  let sizeStore = writable(720);
  onMount(() => {
    canvasStore.set(canvas);
    const w = box.offsetWidth;
    const h = box.offsetHeight;

    sizeStore.set(Math.min(w, h));
  });
  onDestroy(() => {
    canvasStore.set(null);
  });

  let size;
  $: {
      size = $sizeStore;
  }
</script>

<div class="parent" bind:this={box}>
  <canvas
    width={size}
    height={size}
    style="--width: {size}; --height: {size};"
    bind:this={canvas}
  />
</div>

<style>
  .parent {
    width: 100%;
    height: 100%;
    position: relative;
  }

  canvas {
    width: var(--width);
    height: var(--height);
    position: absolute;
  }
</style>
