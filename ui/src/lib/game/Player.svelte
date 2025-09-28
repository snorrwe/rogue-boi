<script>
  import { coreOutput, coreStore, selected } from "@rogueBoi/store.js";

  import Inventory from "./Inventory.svelte";
  import Equipment from "./Equipment.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import Button from "@rogueBoi/Button.svelte";

  let { player, targeting, appMode } = $derived($coreOutput);

  let alive = $derived(player != null);
  let {
    playerHp: hp,
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
      <Button onclick={() => $coreStore.cancelItemUse()}>Cancel item use</Button>
    {/if}

    <h2>Player stats</h2>
    {#if hp != null}
      <div class="flex flex-row align-middle gap-2">
        <ProgressBar
          current={hp.current}
          max={hp.max}
          minColorBg="red"
          minColorFg="white"
          maxColorBg="lime"
          maxColorFg="black"
          midColorBg="orange"
          midColorFg="black"
        />
        {#if levelup}
          <Button onclick={() => $coreStore.setLevelupStat({ ty: "Hp" })}>+</Button>
        {/if}
      </div>
    {/if}
    {#if attack != null}
      <p>
        Attack Power: {attack}
        {#if levelup}
          <Button onclick={() => $coreStore.setLevelupStat({ ty: "Attack" })}>+</Button>
        {/if}
      </p>
    {/if}
    {#if defense != null}
      <p>
        Melee Defense: {defense.meleeDefense}
        {#if levelup}
          <Button onclick={() => $coreStore.setLevelupStat({ ty: "MeleeDefense" })}>+</Button>
        {/if}
      </p>
      {#if defense.ward > 0}
        <p>
          Ward: {defense.ward}
        </p>
      {/if}
    {/if}
    {#if level != null}
      <p>Level: {level}</p>
    {/if}
    {#if currentXp != null}
      <ProgressBar
        current={currentXp}
        max={neededXp}
        minColorBg="#d727d7"
        minColorFg="white"
        maxColorBg="#00b7ff"
        maxColorFg="black"
        midColorBg="#2811ca"
        midColorFg="white"
      />
    {/if}
    <div class="my-2">
      <Button onclick={() => $coreStore.wait()}>Wait</Button>
    </div>
  {:else}
    <p>You died!</p>
    <div class="my-2">
      <Button onclick={() => restartGame()}>Restart</Button>
    </div>
  {/if}
</div>

<style></style>
