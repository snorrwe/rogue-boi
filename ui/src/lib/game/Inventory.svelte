<script>
  import { icons, coreStore, inventory as invs } from "@rogueBoi/store.js";

  let inventory = $derived($invs);

  const useItem = (item) => {
    $coreStore.setSelection(item.id);
  };
</script>

<div class="text-white max-w-60">
  <h2 class="text-xl">Inventory</h2>
  <ul class="grid grid-cols-5 auto-rows-[2.2em] list-none">
    {#each inventory as item}
      <li
        class="h-[2.2rem] w-[2.2rem] cursor-pointer"
        onkeyup={() => useItem(item)}
        onclick={() => useItem(item)}
      >
        <div title={item.description} style="--fill-color: {item.color || 'white'}">
          {@html $icons[item.icon]}
        </div>
      </li>
    {/each}
  </ul>
</div>
