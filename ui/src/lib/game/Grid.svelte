<script>
  import { onDestroy, onMount } from "svelte";
  import { canvasStore } from "@rogueBoi/store.js";

  let box = $state();
  let canvas = $state();
  let size = $state(720);
  /** @type {ResizeObserver} */ let observer;
  onMount(() => {
    canvasStore.set(canvas);
    observer = new ResizeObserver(() => {
      const w = box.offsetWidth;
      const h = box.offsetHeight;

      size = Math.min(w, h);
    });
    observer.observe(box);
  });
  onDestroy(() => {
    canvasStore.set(null);
    if (observer) observer.disconnect();
  });
</script>

<div class="parent" bind:this={box}>
  <canvas width={size} height={size} style="--width: {size}; --height: {size};" bind:this={canvas}
  ></canvas>
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
