<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const user = $derived(data.user);
</script>

<svelte:head>
	<title>{m.home_title()}</title>
</svelte:head>

<div class="overflow-auto">
	<div class="m-8 flex grow flex-col items-center justify-center">
		<div class="card w-md">
			<p class="greeting">
				{m.home_welcome({ name: user.info?.name ?? user.username ?? 'user' })}
			</p>
		</div>
	</div>
</div>
