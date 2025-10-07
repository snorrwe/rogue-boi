<script>
  import { icons, inventory } from "@rogueBoi/store.js";
  import ProgressBar from "./ProgressBar.svelte";
  import Button from "../Button.svelte";

  let { selected, core, targetingMode } = $props();

  let icon = $derived($icons[selected.icon]);
  $effect(() => {
    console.assert(icon != null, "Highlight icon not found", icon);
  });
  let droppable = $derived(
    selected.item && !selected.equipped && $inventory.some((i) => i.id == selected.id)
  );
  let unequippable = $derived(selected.equipped);

  const useItem = (item) => () => {
    core.useItem(item.id);
  };

  const dropItem = (item) => () => {
    core.dropItem(item.id);
  };

  const unequipItem = (item) => () => {
    core.unequipItem(item.id);
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
  <Button onclick={useItem(selected)}>Use</Button>
{/if}
{#if selected.equipable}
  <Button onclick={useItem(selected)}>Equip</Button>
{/if}
{#if droppable}
  <Button onclick={dropItem(selected)}>Drop</Button>
{/if}
{#if unequippable}
  <Button onclick={unequipItem(selected)}>Unequip</Button>
{/if}
{#if selected.targetable && targetingMode}
  <Button onclick={target(selected)}>Target</Button>
{/if}

<style>
  .icon {
    height: 3.2rem;
    width: 3.2rem;
  }
</style>
