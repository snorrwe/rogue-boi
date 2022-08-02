<script>
	import { coreStore } from '@rogueBoi/store.js';

	function clearSave() {
		localStorage.removeItem('save');
		coreStore.set(null);
		window.location.reload();
	}
</script>

<button on:click={clearSave}>Clear save game</button>
