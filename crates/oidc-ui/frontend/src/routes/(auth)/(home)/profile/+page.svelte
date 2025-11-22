<script lang="ts">
	import type { PageProps } from './$types';
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import UsernameForm from './UsernameForm.svelte';
	import PasswordForm from './PasswordForm.svelte';
	import InfoForm from './InfoForm.svelte';

	let { data }: PageProps = $props();

	let user = $derived(data.user) as User;
</script>

<svelte:head>
	<title>Profile</title>
</svelte:head>

<div class="space-y-4">
	<h1 class="text-2xl font-semibold">Profile</h1>

	<section class="card">
		<h2 class="mb-2 text-lg font-medium">Account</h2>
		<p class="text-sm"><strong>Username:</strong> <span class="ml-2">{user.username}</span></p>
		<p class="text-sm">
			<strong>Email:</strong>
			<span class="ml-2">{user.email?.email ?? user.emails?.[0]?.email ?? '—'}</span>
		</p>
		<p class="text-sm">
			<strong>Active:</strong> <span class="ml-2">{user.active ? 'Yes' : 'No'}</span>
		</p>
		<p class="text-sm">
			<strong>Created:</strong>
			<span class="ml-2">{new Date(user.createdAt).toLocaleString()}</span>
		</p>
	</section>

	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<UsernameForm {user} on:update={(e) => (user = e.detail)} />
		<PasswordForm {user} on:update={(e) => (user = e.detail)} />
	</div>

	<InfoForm {user} on:update={(e) => (user = e.detail)} />
</div>
