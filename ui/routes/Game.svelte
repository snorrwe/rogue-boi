<script>
	import {
		canvasStore,
		coreStore,
		coreOutput,
		inventory,
		equipment,
		selected
	} from '@rogueBoi/store.js';
	import Grid from '@rogueBoi/game/Grid.svelte';
	import Highlight from '@rogueBoi/game/Highlight.svelte';
	import Log from '@rogueBoi/game/Log.svelte';
	import Player from '@rogueBoi/game/Player.svelte';

	let core;
	coreStore.subscribe((c) => (core = c));

	let last = new Date().getTime();

	canvasStore.subscribe((canvas) => core.setCanvas(canvas));

	const onKey = (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	};

	const gameLoop = () => {
		if (core) {
			const now = new Date().getTime();

			core.tick(now - last);

			const output = core.getOutput();
			coreOutput.set(output);
			inventory.set(core.getInventory());
			equipment.set(core.getEquipment());
			last = now;
			if (output && output.selected) {
				// TODO: include the entity information in selected field
				selected.set(core.fetchEntity(output.selected));
			}
		}
		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<svelte:window on:keydown={onKey} />

<main>
	{#if core != null}
		<div class="content">
			<div>
				<Grid />
			</div>
			<div class="log">
				<div>Dungeon floor: {$coreOutput.dungeonLevel}</div>
				<h2>Log</h2>
				<Log log={$coreOutput.log} />
			</div>
			<div class="game-ui">
				{#if $selected != null}
					<Highlight targetingMode={$coreOutput.targeting} selected={$selected} {core} />
				{/if}
				<div>
					<Player />
				</div>
			</div>
		</div>
	{/if}
</main>

<style>
	main {
		padding: 2em;
		margin: 0 auto;
		color: white;
		min-width: 0px;
	}

	.content {
		display: grid;
		grid-template-columns: 2fr repeat(2, 1fr);
		column-gap: 2rem;
	}

	.log {
		max-height: 100%;
		overflow: auto;
	}

	.game-ui {
		color: white;
		text-align: left;
	}
</style>
