<script>
  import { icons } from "@rogueBoi/store.js";

  let { item } = $props();
</script>

<div class="border-2 p-4 align-middle content-center justify-center">
  {#if item}
    <div class="flex flex-col">
      <span style="--fill-color: {item.color || 'white'}">
        {@html $icons[item.icon]}
      </span>
      <span>
        {item.tag}
      </span>
      <span>
        Cost: {item.cost}
      </span>
    </div>
  {:else}
    Empty
  {/if}
</div>
