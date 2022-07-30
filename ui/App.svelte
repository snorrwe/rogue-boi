<script>
	import { onMount } from 'svelte';
	import { fetchIcon, coreStore } from '@rogueBoi/store.js';
	import Menu from './routes/Menu.svelte';
	import Game from './routes/Game.svelte';

	export let core;
	let page = Menu;

	onMount(() => {
		core.icons().forEach((icon) => {
			fetchIcon({ src: `icons/${icon}.svg`, name: icon });
		});
		coreStore.set(core);
		routeChange();
	});

	document.addEventListener('keydown', (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	});

	const routeChange = () => {
		let factory =
			{
				'#game': () => Game,
				'#newgame': () => {
					if (core) {
						core.restart();
						return Game;
					}
				},
				'#menu': () => Menu
			}[location.hash] || (() => Menu);
		page = factory();
	};
</script>

<svelte:window on:hashchange={routeChange} />

<main>
	<header>
		<a href="#menu">Back to menu</a>
	</header>
	<svelte:component this={page} />
</main>

<style>
	main {
		padding: 2em;
		margin: 0 auto;
		color: white;
		min-width: 0px;
	}
</style>
