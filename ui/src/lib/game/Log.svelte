<script>
  let { log } = $props();
</script>

{#if log}
  <ul>
    {#each log as line}
      <li>{@html line}</li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    list-style: none;
  }
</style>
