<script>
    export let core;
    export let grid;
</script>

<div class="grid" style="--cols: {grid.dims.x}; --rows: {grid.dims.y}">

{#each grid.grid as item}
    <div class="grid-item">
        {#if item.id }
            <img src="/icons/ffffff/transparent/1x1/{core.get_icon(item.id)}" />
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

