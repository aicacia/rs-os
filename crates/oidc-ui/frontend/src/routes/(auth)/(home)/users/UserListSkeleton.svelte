<script lang="ts">
	// Simple skeleton placeholders for list/table
</script>

<div class="space-y-2">
	{#each Array.from({ length: 6 }) as _}
		<div class="h-10 w-full animate-pulse rounded-md bg-gray-200 dark:bg-gray-700"></div>
	{/each}
</div>
