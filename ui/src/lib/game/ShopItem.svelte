<script>
  import { icons, coreStore } from "@rogueBoi/store.js";

  let { item, idx } = $props();
  let core = $derived($coreStore);

  let buying = $state(false);

  function purchase() {
    try {
      core.buyItem(idx);
    } catch (err) {
      console.error("Failed to buy item", err);
    }
  }
</script>

<div class="border-2 p-4 align-middle content-center justify-center">
  {#if item}
    <button
      class="flex flex-col cursor-pointer"
      onclick={() => {
        buying = !buying;
      }}
    >
      <span style="--fill-color: {item.color || 'white'}">
        {@html $icons[item.icon]}
      </span>
      <span>
        {item.tag}
      </span>
      <span>
        Cost: {item.cost}
      </span>
    </button>
    {#if buying}
      <button class="button" onclick={purchase}>Purchase</button>
      <button
        class="button"
        onclick={() => {
          buying = !buying;
        }}
      >
        Cancel
      </button>
    {/if}
  {:else}
    Empty
  {/if}
</div>

<style>
  .button {
    cursor: pointer;
    border-style: solid;
    border-width: 2px;
    padding: 8px;
  }
</style>
