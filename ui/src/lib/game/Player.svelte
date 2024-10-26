<script>
  import { coreOutput, coreStore, selected } from "@rogueBoi/store.js";

  import Inventory from "./Inventory.svelte";
  import Equipment from "./Equipment.svelte";
  import Hp from "./Hp.svelte";

  let { player, targeting, appMode } = $derived($coreOutput);

  let alive = $derived(player != null);
  let {
    playerHp: hp,
    playerPos: pos,
    playerAttack: attack,
    currentXp,
    neededXp,
    level,
    defense
  } = $derived(player ? player : {});
  let levelup = $derived(appMode && appMode.ty == "Levelup");

  function restartGame() {
    selected.set(null);
    $coreStore.restart();
  }
</script>

<div>
  {#if alive}
    <Equipment />
    <Inventory />

    {#if targeting}
      <button onclick={() => $coreStore.cancelItemUse()}>Cancel item use</button>
    {/if}

    <h2>Player stats</h2>
    {#if hp != null}
      <div>
        {#if levelup}
          <button onclick={() => $coreStore.setLevelupStat({ ty: "Hp" })}>+</button>
        {/if}
        <Hp currentHp={hp.current} maxHp={hp.max} />
      </div>
    {/if}
    {#if attack != null}
      <p>
        {#if levelup}
          <button onclick={() => $coreStore.setLevelupStat({ ty: "Attack" })}>+</button>
        {/if}
        Attack Power: {attack}
      </p>
    {/if}
    {#if defense != null}
      <p>
        {#if levelup}
          <button onclick={() => $coreStore.setLevelupStat({ ty: "MeleeDefense" })}>+</button>
        {/if}
        Melee Defense: {defense.meleeDefense}
      </p>
      {#if defense.ward > 0}
        <p>
          Ward: {defense.ward}
        </p>
      {/if}
    {/if}
    {#if pos != null}
      <p id="player-pos">
        Position: {pos.x}, {pos.y}
      </p>
    {/if}
    {#if level != null}
      <p>Level: {level}</p>
    {/if}
    {#if currentXp != null}
      <p>Experience: {currentXp} / {neededXp}</p>
    {/if}
    <button onclick={() => $coreStore.wait()}>Wait</button>
  {:else}
    <p>You died!</p>
    <button onclick={() => restartGame()}>Restart</button>
  {/if}
</div>

<style></style>
