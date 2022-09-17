<script>
	import { coreOutput, coreStore, selected } from '@rogueBoi/store.js';

	import Inventory from './Inventory.svelte';
	import Equipment from './Equipment.svelte';

	$: ({
		player,
		targeting,
		appMode: { ty: appMode }
	} = $coreOutput);

	$: alive = player != null;
	$: ({
		playerHp: hp,
		playerPos: pos,
		playerAttack: attack,
		currentXp,
		neededXp,
		level,
		defense
	} = player ?? {});
	$: levelup = appMode == 'Levelup';

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
			<button on:click={() => $coreStore.cancelItemUse()}>Cancel item use</button>
		{/if}

		<h2>Player stats</h2>
		{#if hp != null}
			<p id="player-hp">
				{#if levelup}
					<button on:click={() => $coreStore.setLevelupStat({ ty: 'Hp' })}>+</button>
				{/if}
				Health: {hp.current} / {hp.max}
			</p>
		{/if}
		{#if attack != null}
			<p>
				{#if levelup}
					<button on:click={() => $coreStore.setLevelupStat({ ty: 'Attack' })}>+</button>
				{/if}
				Attack Power: {attack}
			</p>
		{/if}
		{#if defense != null}
			<p>
				{#if levelup}
					<button on:click={() => $coreStore.setLevelupStat({ ty: 'MeleeDefense' })}>+</button>
				{/if}
				Melee Defense: {defense.meleeDefense}
			</p>
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
		<button on:click={() => $coreStore.wait()}>Wait</button>
	{/if}

	{#if !alive}
		<p>You died!</p>
		<button on:click={() => restartGame()}>Restart</button>
	{/if}
</div>

<style></style>
