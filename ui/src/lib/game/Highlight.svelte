<script>
  import { icons } from "@rogueBoi/store.js";
  import Hp from "./Hp.svelte";

  export let selected;
  export let core;
  export let targetingMode;

  $: icon = $icons[selected.icon];

  console.assert(icon != null, "Highlight icon not found", icon);

  const useItem = (item) => () => {
    core.useItem(item.id);
  };

  const dropItem = (item) => () => {
    core.dropItem(item.id);
  };

  const target = (item) => () => {
    core.setTarget(item.id);
  };
</script>

<div class="icon" style="--fill-color: {selected.color || 'white'}">
  {@html icon}
</div>
{#if selected.name}
  <h3>
    {selected.name}
  </h3>
{/if}
{#if selected.description}
  <div>
    {selected.description}
  </div>
{/if}
{#if selected.hp}
  <Hp currentHp={selected.hp.current} maxHp={selected.hp.max} />
{/if}
{#if selected.melee}
  <div>Melee Power: {selected.melee.power}</div>
  <div>Melee Skill: {selected.melee.skill}</div>
{/if}
{#if selected.defense}
  <div>Melee Defense: {selected.defense.meleeDefense}</div>
{/if}
{#if selected.ranged}
  <div>Ranged Power: {selected.ranged.power}</div>
  <div>Ranged Skill: {selected.ranged.skill}</div>
{/if}
{#if selected.usable}
  <button on:click={useItem(selected)}>Use</button>
{/if}
{#if selected.equipable}
  <button on:click={useItem(selected)}>Equip</button>
{/if}
{#if selected.item && !selected.equipped}
  <button on:click={dropItem(selected)}>Drop</button>
{/if}
{#if selected.targetable && targetingMode}
  <button on:click={target(selected)}>Target</button>
{/if}

<style>
  .icon {
    height: 3.2rem;
    width: 3.2rem;
  }
</style>
