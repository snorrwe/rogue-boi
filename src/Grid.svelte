<script>
    export let core;
    export let grid;

    const isItemVisible = (i, grid) => {
        let y = Math.floor(i / grid.grid.dims.x);
        let x = i - y * grid.grid.dims.x;

        x += grid.offset.x;
        y += grid.offset.y;
        return core.visible({x,y})
    }
</script>

<div class="grid" style="--cols: {grid.grid.dims.x}; --rows: {grid.grid.dims.y}">

{#each grid.grid.data as item, i}
    <div class="grid-item" >
        <div class:grid_visible="{isItemVisible(i, grid)}">
            {#if item.id}
                <img src="/icons/ffffff/transparent/1x1/{core.get_icon(item.id)}" />
            {:else}
                <div class="floor"></div>
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

    .floor {
        height: 2rem;
        width: 2rem;
    }
</style>

