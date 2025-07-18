<script>
  import { icons, inventory } from "@rogueBoi/store.js";
  import ProgressBar from "./ProgressBar.svelte";

  let { selected, core, targetingMode } = $props();

  let icon = $derived($icons[selected.icon]);
  $effect(() => {
    console.assert(icon != null, "Highlight icon not found", icon);
  });
  let droppable = $derived(
    selected.item && !selected.equipped && $inventory.some((i) => i.id == selected.id)
  );

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
  <ProgressBar
    current={selected.hp.current}
    max={selected.hp.max}
    minColorBg="red"
    minColorFg="white"
    maxColorBg="lime"
    maxColorFg="black"
    midColorBg="orange"
    midColorFg="black"
  />
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
  <button onclick={useItem(selected)}>Use</button>
{/if}
{#if selected.equipable}
  <button onclick={useItem(selected)}>Equip</button>
{/if}
{#if droppable}
  <button onclick={dropItem(selected)}>Drop</button>
{/if}
{#if selected.targetable && targetingMode}
  <button onclick={target(selected)}>Target</button>
{/if}

<style>
  .icon {
    height: 3.2rem;
    width: 3.2rem;
  }
</style>
