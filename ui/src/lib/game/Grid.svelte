<script>
  import { canvasStore } from "@rogueBoi/store.js";

  let canvas = $state(null);
  let box = $state(null);
  let size = $state(720);
  /** @type {ResizeObserver} */ let observer;

  $effect(() => {
    canvasStore.set(canvas);
    observer = new ResizeObserver(() => {
      const w = box.offsetWidth;
      const h = box.offsetHeight;

      size = Math.min(w, h);
    });
    observer.observe(box);
    return () => {
      canvasStore.set(null);
      observer.disconnect();
    };
  });
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
