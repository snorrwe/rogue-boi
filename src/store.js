import { writable } from 'svelte/store';

export const canvasStore = writable(null);
export const iconStore = writable({});

export const fetchIcon = ({ name, src }) =>
	fetch(src)
		.then((r) => r.text())
		.then((data) => {
			iconStore.update((ic) => ({
				[name]: data,
				...ic
			}));
			return data;
		});
