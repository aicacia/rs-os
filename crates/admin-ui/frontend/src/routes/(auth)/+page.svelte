<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { resolve } from '$app/paths';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const name = data.user.nickname ?? data.user.preferredUsername; 
</script>

<svelte:head>
	<title>{m.home_title ? m.home_title() : 'Home'}</title>
</svelte:head>

<div class="card mb-4">
	<h1 class="text-2xl font-semibold">
		{m.home_welcome ? m.home_welcome({ name }) : `Welcome, ${name}`}
	</h1>
	<p class="text-muted-foreground mt-2 text-sm">
		Manage your profile and administration from this dashboard.
	</p>
</div>

<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
	<a href={resolve('/profile')} class="card">
		<h2 class="font-medium">Profile</h2>
		<p class="text-muted-foreground text-sm">View and edit your profile.</p>
	</a>

	<a href={resolve('/clients')} class="card">
		<h2 class="font-medium">Clients</h2>
		<p class="text-muted-foreground text-sm">Create, read, update and delete clients.</p>
	</a>

	<a href={resolve('/users')} class="card">
		<h2 class="font-medium">Users</h2>
		<p class="text-muted-foreground text-sm">Manage application users and permissions.</p>
	</a>

	<a href={resolve('/config')} class="card">
		<h2 class="font-medium">Application Config</h2>
		<p class="text-muted-foreground text-sm">
			Edit application-level configuration (requires permissions).
		</p>
	</a>
</div>
