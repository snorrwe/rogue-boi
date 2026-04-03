<script>
  import { coreOutput, icons } from "@rogueBoi/store.js";

  let shop = $derived($coreOutput?.shop);
  let inventory = $derived(shop?.inventory);
</script>

<h1 class="text-3xl">Shop</h1>

<div class="mt-8 grid grid-cols-4 align-middle gap-4">
  {#each inventory ?? [] as item}
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
  {/each}
</div>
