<script>
    export let hp;
    export let pos;
</script>

<div>
{#if hp != null}
    <p id="player-hp">
        HP: {hp.current} / {hp.max}
    </p>
{/if}
{#if pos != null}
    <p id="player-pos">
        Position: {pos.x}, {pos.y}
    </p>
{/if}
</div>

<style>
    div {
        color: white;
        text-align: left;
        margin: 2em;
    }
</style>
