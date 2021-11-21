<script>
    export let core;
    export let grid;

    const isItemVisible = (i, grid) => {
        let y = Math.floor(i / grid.dims.x);
        let x = i - y * grid.dims.x;
        return core.visible({x,y})
    }
</script>

<div class="grid" style="--cols: {grid.dims.x}; --rows: {grid.dims.y}">

{#each grid.data as item, i}
    <div class="grid-item" >
        <div class:grid_visible="{isItemVisible(i, grid)}">
            {#if item.id}
                <img src="/icons/ffffff/transparent/1x1/{core.get_icon(item.id)}" />
            {/if}
        </div>
    </div>
{/each}

</div>

<style>
    .grid {
        display: grid;
        grid-template-columns: repeat(var(--cols), 2em);
        grid-auto-rows: 2em;
    }

    .grid-item {
        color: #ff3e00;
    }

    .grid-item .grid_visible {
        color: yellow;
        background: yellow;
    }
</style>

