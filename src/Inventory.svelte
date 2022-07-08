<script>
	export let inventory;
</script>

<div>
	<ul>
		{#each inventory ?? [] as item}
			<li>
				<img src="/icons/ffffff/transparent/1x1/{item[1]}" />
			</li>
		{/each}
	</ul>
</div>

<style>
	div {
		color: white;
	}
</style>
