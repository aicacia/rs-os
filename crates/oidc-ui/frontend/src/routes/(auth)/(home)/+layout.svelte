<script lang="ts">
	import Sidebar from '$lib/common/components/Sidebar.svelte';
	import { resolve } from '$app/paths';
	import type { LayoutProps } from './$types';
	import { hasPermission } from '$lib/common/state/currentUser.svelte';
	import { Permission } from '$lib/common/openapi/oidc';

	let { data, children }: LayoutProps = $props();

	const user = $derived(data.user);
</script>

<Sidebar>
	{#snippet nav()}
		<li class="m-0">
			<a class="flex grow p-2" href={resolve('/')}>Home</a>
		</li>
		<li class="m-0">
			<a class="flex grow p-2" href={resolve('/profile')}>Profile</a>
		</li>
		{#if hasPermission(user, Permission.Admin)}
			<li class="m-0">
				<a class="flex grow p-2" href={resolve('/config')}>Application Config</a>
			</li>
		{/if}
		{#if hasPermission(user, Permission.ClientWrite) || hasPermission(user, Permission.ClientRead)}
			<li class="m-0">
				<a class="flex grow p-2" href={resolve('/clients')}>Clients</a>
			</li>
		{/if}
		{#if hasPermission(user, Permission.UserWrite) || hasPermission(user, Permission.UserRead)}
			<li class="m-0">
				<a class="flex grow p-2" href={resolve('/users')}>Users</a>
			</li>
		{/if}
	{/snippet}
	{@render children()}
</Sidebar>
