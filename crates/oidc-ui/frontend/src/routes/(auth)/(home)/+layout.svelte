<script lang="ts">
	import type { LayoutProps } from './$types';
	import Navbar from '$lib/common/components/Navbar.svelte';

	let { data, children }: LayoutProps = $props();

	const user = $derived(data.user);
</script>

<Navbar {user}>
	{@render children()}
</Navbar>
