import App from './App.svelte';
import wasmInit, { initCore } from 'rogue-boi-core';

const init = async () => {
	await wasmInit();
	const core = initCore();
	return new App({
		target: document.body,
		props: {
			core
		}
	});
};

init();
