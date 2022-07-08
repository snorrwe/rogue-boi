<script>
	export let inventory;
</script>

<div>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item">
				<div title={item[1][1]}>
					<img src="/icons/ffffff/transparent/1x1/{item[1][0]}" />
				</div>
			</li>
		{/each}
	</ul>
</div>

<style>
	div {
		color: white;
	}

	.item {
		height: 2.2rem;
		width: 2.2rem;
	}
</style>
