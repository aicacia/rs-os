<script lang="ts" module>
	export interface ClientInfoLogoProps {
		client: Partial<ClientInfo>;
	}
</script>

<script lang="ts">
	import { getAverageLuminance } from '$lib/common/util/canvas';
	import type { ClientInfo } from './+page.svelte';

	let { client }: ClientInfoLogoProps = $props();

	let clientLogoUriElement = $state<HTMLImageElement | null>();
	let isClientLogoDark = $state(true);

	$effect(() => {
		if (clientLogoUriElement) {
			getAverageLuminance(clientLogoUriElement).then((luminance) => {
				isClientLogoDark = luminance < 200;
			});
		}
	});
</script>

{#if client.logoUri}
	<img
		bind:this={clientLogoUriElement}
		src={client.logoUri}
		alt={`${client.name} logo`}
		crossorigin="anonymous"
		class={{
			'h-24 w-24 rounded-full p-4': true,
			'bg-black': !isClientLogoDark,
			'bg-white': isClientLogoDark
		}}
	/>
{/if}
