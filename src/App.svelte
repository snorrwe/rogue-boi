<script>
	import { canvasStore } from './store.js';
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
	import Highlight from './Highlight.svelte';
	import Log from './Log.svelte';
	import Player from './Player.svelte';
	import { onMount } from 'svelte';
	import { fetchIcon } from './store.js';

	export let core;

	onMount(() => {
		core.icons().forEach((icon) => {
			fetchIcon({ src: `icons/${icon}.svg`, name: icon });
		});
	});

	const coreOutput = writable(core.getOutput());
	let selected = writable(null);
	coreOutput.subscribe((c) => selected.set(c && c.selected && core.fetchEntity(c.selected)));
	const inventory = writable(core.getInventory());
	let last = new Date().getTime();

	canvasStore.subscribe((canvas) => core.setCanvas(canvas));

	document.addEventListener('keydown', (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	});

	const gameLoop = () => {
		const now = new Date().getTime();

		core.tick(now - last);

		coreOutput.set(core.getOutput());
		inventory.set(core.getInventory());
		last = now;
		if ($selected && $selected.id) {
			selected.set(core.fetchEntity($selected.id));
		}

		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<main>
	{#if core != null}
		<div class="content">
			<div>
				<Grid />
			</div>
			<div>
				{#if $selected != null}
					<Highlight selected={$selected} {core} />
				{/if}
				<div>
					<Player
						inventory={$inventory}
						alive={$coreOutput.player != null}
						hp={$coreOutput.player && $coreOutput.player.playerHp}
						pos={$coreOutput.player && $coreOutput.player.playerPosition}
						attack={$coreOutput.player && $coreOutput.player.playerAttack}
						{core}
					/>
				</div>
			</div>
			<div class="log">
				<h2>Logs</h2>
				<Log log={$coreOutput.log} />
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
		grid-template-columns: minmax(0, 2fr) repeat(2, 1fr);
	}

	.log {
		max-height: 100%;
		overflow: auto;
	}
</style>
