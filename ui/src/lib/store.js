import { writable } from "svelte/store";

export const coreStore = writable(null);
export const coreOutput = writable({});
export const canvasStore = writable(null);
export const icons = writable({});
export const inventory = writable([]);
export const equipment = writable(null);
export const selected = writable(null);

export const fetchIcon = ({ name, src }) =>
    fetch(src)
        .then((r) => r.text())
        .then((data) => {
            icons.update((ic) => ({
                [name]: data,
                ...ic
            }));
            return data;
        });
