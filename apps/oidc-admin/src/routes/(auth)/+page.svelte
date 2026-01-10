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

<div class="space-y-4">
	<div class="card">
		<h1>{m.home_welcome ? m.home_welcome({ name }) : `Welcome, ${name}`}</h1>
		<p>Manage your profile and administration from this dashboard.</p>
	</div>

	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
		<a href={resolve('/profile')} class="card default">
			<h3>Profile</h3>
			<p>View and edit your profile.</p>
		</a>

		<a href={resolve('/clients')} class="card default">
			<h3>Clients</h3>
			<p>Create, read, update and delete clients.</p>
		</a>

		<a href={resolve('/users')} class="card default">
			<h3>Users</h3>
			<p>Manage application users and permissions.</p>
		</a>

		<a href={resolve('/config')} class="card default">
			<h3>Application Config</h3>
			<p>Edit application-level configuration (requires permissions).</p>
		</a>
	</div>
</div>
