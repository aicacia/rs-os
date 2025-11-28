<script lang="ts">
	import type { PageProps } from './$types';
	import { Search, Plus } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import UserTable from './UserTable.svelte';
	import UserCard from './UserCard.svelte';
	import UserListSkeleton from './UserListSkeleton.svelte';
	import * as m from '$lib/paraglide/messages/_index.js';
	import { getCurrentUser, hasPermissions } from '$lib/common/state/currentUser.svelte';
	import { Permission, type User as OUser } from '$lib/common/openapi/oidc/models';

	let { data }: PageProps = $props();
	let query = $state('');
	let users = $state<OUser[] | null>(null);
	let canCreate = $state(false);

	$effect(() => {
		users = data.users ?? [];
	});

	$effect(() => {
		getCurrentUser().then((current) => {
			canCreate = !!(current && hasPermissions(current, [Permission.UserWrite]));
		});
	});

	const filtered = $derived.by(() => {
		if (!users) return [];
		if (!query) return users;
		const q = query.toLowerCase();
		return users.filter((u) => String(u.id).includes(q) || u.username.toLowerCase().includes(q));
	});

	function onCreate() {
		goto(resolve('/(auth)/(home)/users/create'));
	}
	function onEdit(u: OUser) {
		goto(resolve('/(auth)/(home)/users/[id]/edit', { id: u.id.toString() }));
	}
	function onView(u: OUser) {
		goto(resolve('/(auth)/(home)/users/[id]', { id: u.id.toString() }));
	}
</script>

<svelte:head>
	<title>{(m as any).users_title()}</title>
</svelte:head>

<section class="space-y-4 p-4">
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-semibold">{(m as any).users_title()}</h1>
		{#if canCreate}
			<button class="btn primary flex items-center gap-2" onclick={onCreate}>
				<Plus class="h-4 w-4" />
				{(m as any).users_create_label()}
			</button>
		{/if}
	</div>

	<div class="relative max-w-xl">
		<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500" />
		<input
			class="w-full pl-9"
			bind:value={query}
			placeholder={(m as any).users_search_placeholder()}
		/>
	</div>

	{#if users === null}
		<UserListSkeleton />
	{:else if users.length === 0}
		<p class="text-gray-600 dark:text-gray-400">{(m as any).users_empty()}</p>
	{:else if filtered.length === 0}
		<p class="text-gray-600 dark:text-gray-400">{(m as any).users_no_match()}</p>
	{:else}
		<div class="hidden md:block">
			<UserTable users={filtered} {onEdit} onDelete={onView} />
		</div>
		<div class="grid gap-3 md:hidden">
			{#each filtered as u}
				<UserCard user={u} {onEdit} onDelete={onView} />
			{/each}
		</div>
	{/if}
</section>
