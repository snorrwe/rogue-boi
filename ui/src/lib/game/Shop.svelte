<script>
  import { coreOutput } from "@rogueBoi/store.js";
  import ShopItem from "./ShopItem.svelte";

  let shop = $derived($coreOutput?.shop);
  let inventory = $derived(shop?.inventory);
</script>

<h1 class="text-3xl">Shop</h1>

<div class="mt-8 grid grid-cols-4 align-middle gap-4">
  {#each inventory ?? [] as item, idx}
    <ShopItem {item} {idx} />
  {/each}
</div>
