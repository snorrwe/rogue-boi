<script>
    export let grid;

    import {Tile} from "./entities.js";
</script>

<div class="grid" style="--cols: {grid.dims.x}; --rows: {grid.dims.y}">

{#each grid.grid as item}
    <div class="grid-item">
{#if Tile[item?.ty]}
        <img src="/icons/ffffff/transparent/1x1/{Tile[item.ty]}" />
{/if}
    </div>
{/each}

</div>

<style>
    div {
        color: #ff3e00;
    }

    .grid {
        display: grid;
        grid-template-columns: repeat(var(--cols), 2em);
        grid-auto-rows: 2em;
    }

    .grid-item {
        min-height: 1em;
        min-width: 1em;
    }
</style>

