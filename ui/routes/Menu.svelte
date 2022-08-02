<script>
	import { coreOutputStore } from '@rogueBoi/store.js';

	let alive = false;
	coreOutputStore.subscribe((c) => (alive = c && c.player != null));
</script>

<nav class="menu">
	<ul>
		{#if alive}
			<li><a href="#game">Continue</a></li>
		{/if}
		<li><a href="#newgame">New Game</a></li>
		<li><a href="#options">Options</a></li>
	</ul>
</nav>

<style>
	.menu {
		width: 50%;
		margin: 0 auto;
	}

	.menu ul {
		list-style: none;
	}

	li {
		margin: 1rem auto;
	}
</style>
