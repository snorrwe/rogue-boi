<script>
  import { icons, coreStore, equipment } from "@rogueBoi/store.js";

  let armor = $derived($equipment && $equipment.get("armor"));
  let weapon = $derived($equipment && $equipment.get("weapon"));

  const selectItem = (item) => {
    $coreStore.setSelection(item.get("id"));
  };
</script>

<div>
  <h2 class="text-xl">Equipment</h2>
  <ul>
    {#if armor != null}
      <li
        role="menuitem"
        class="item"
        onclick={() => selectItem(armor)}
        onkeypress={() => selectItem(armor)}
      >
        <div title={armor.get("description")} style="--fill-color: {armor.get('color') || 'white'}">
          {@html $icons[armor.get("icon")]}
        </div>
      </li>
    {/if}
    {#if weapon != null}
      <li
        role="menuitem"
        class="item"
        onclick={() => selectItem(weapon)}
        onkeypress={() => selectItem(weapon)}
      >
        <div
          title={weapon.get("description")}
          style="--fill-color: {weapon.get('color') || 'white'}"
        >
          {@html $icons[weapon.get("icon")]}
        </div>
      </li>
    {/if}
  </ul>
</div>

<style>
  ul {
    display: grid;
    grid-template-columns: repeat(2, 2.2em);
    grid-auto-rows: 2.2em;
    list-style: none;
  }

  .item {
    height: 2.2rem;
    width: 2.2rem;
    cursor: pointer;
  }
</style>
