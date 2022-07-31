<script>
	import { canvasStore, coreStore, coreOutputStore } from '@rogueBoi/store.js';
	import { writable } from 'svelte/store';
	import Grid from '@rogueBoi/game/Grid.svelte';
	import Highlight from '@rogueBoi/game/Highlight.svelte';
	import Log from '@rogueBoi/game/Log.svelte';
	import Player from '@rogueBoi/game/Player.svelte';

	let core;
	coreStore.subscribe((c) => (core = c));

	let selected = writable(null);
	coreOutputStore.subscribe((c) => selected.set(c && c.selected && core.fetchEntity(c.selected)));
	const inventory = writable(core.getInventory());
	let last = new Date().getTime();

	canvasStore.subscribe((canvas) => core.setCanvas(canvas));

	const onKey = (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	};

	function letsgoboi() {
		let pl = core.save();
		console.log(pl.length, JSON.parse(pl));
		core.load(pl);
		// hack: re-initialize core
		canvasStore.update((canvas) => {
			return canvas;
		});
	}

	const gameLoop = () => {
		if (core) {
			const now = new Date().getTime();

			core.tick(now - last);

			coreOutputStore.set(core.getOutput());
			inventory.set(core.getInventory());
			last = now;
			if ($selected && $selected.id) {
				selected.set(core.fetchEntity($selected.id));
			}
		}
		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<svelte:window on:keydown={onKey} />

<main>
	{#if core != null}
		<div>
			<button on:click={letsgoboi}>Save Load test</button>
		</div>
		<div class="content">
			<div>
				<Grid />
			</div>
			<div class="log">
				<h2>Log</h2>
				<Log log={$coreOutputStore.log} />
			</div>
			<div class="game-ui">
				{#if $selected != null}
					<Highlight targetingMode={$coreOutputStore.targeting} selected={$selected} {core} />
				{/if}
				<div>
					<Player
						inventory={$inventory}
						alive={$coreOutputStore.player != null}
						hp={$coreOutputStore.player && $coreOutputStore.player.playerHp}
						pos={$coreOutputStore.player && $coreOutputStore.player.playerPosition}
						attack={$coreOutputStore.player && $coreOutputStore.player.playerAttack}
						targeting={$coreOutputStore.targeting}
						{core}
					/>
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
