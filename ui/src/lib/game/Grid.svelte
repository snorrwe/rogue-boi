<script>
  import { canvasStore } from "@rogueBoi/store.js";

  let canvas = $state(null);

  $effect(() => {
    canvasStore.set(canvas);
    return () => {
      canvasStore.set(null);
    };
  });
</script>

<canvas width="600" height="600" bind:this={canvas} />
