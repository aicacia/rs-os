<script lang="ts">
	import type { PageProps } from './$types';
	import { Search, Plus, ArrowLeft } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import UserTable from './UserTable.svelte';
	import UserCard from './UserCard.svelte';
	import UserListSkeleton from './UserListSkeleton.svelte';
	import { m } from '$lib/paraglide/messages';
	import { hasPermissions } from '$lib/common/state/user.svelte';
	import { Permission, type User as OUser } from '$lib/common/openapi/admin/models';

	let { data }: PageProps = $props();
	let query = $state('');
	let users = $state<OUser[] | null>(null);
	let canCreate = $state(false);

	$effect(() => {
		users = data.users ?? [];
	});

	$effect(() => {
		canCreate = hasPermissions(data.user, [Permission.UserWrite]);
	});

	const filtered = $derived.by(() => {
		if (!users) return [];
		if (!query) return users;
		const q = query.toLowerCase();
		return users.filter((u) => String(u.id).includes(q) || u.username.toLowerCase().includes(q));
	});

	function onCreate() {
		goto(resolve('/(auth)/users/create'));
	}
	function onEdit(u: OUser) {
		goto(resolve('/(auth)/users/[id]/edit', { id: u.id.toString() }));
	}
	function onView(u: OUser) {
		goto(resolve('/(auth)/users/[id]', { id: u.id.toString() }));
	}
</script>

<svelte:head>
	<title>{m.users_title()}</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex gap-4 flex-row items-center justify-between">
			<div class="flex gap-4 flex-row items-center">
				<a href={resolve('/')}>
					<ArrowLeft />
				</a>
				<h2 class="m-0">{m.users_title()}</h2>
			</div>
			{#if canCreate}
				<button
					class="btn primary flex items-center gap-2"
					onclick={onCreate}
				>
					<Plus class="h-5 w-5" />
					{m.users_create_label()}
				</button>
			{/if}
		</div>

		<div class="mt-4">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400" />
				<input
					type="text"
					placeholder={m.users_search_placeholder()}
					bind:value={query}
					class="w-full pl-10"
				/>
			</div>
		</div>
	</section>

	<section class="card">
		{#if users === null}
			<UserListSkeleton />
		{:else if users.length === 0}
			<p class="text-gray-600 dark:text-gray-400">{m.users_empty()}</p>
		{:else if filtered.length === 0}
			<p class="text-gray-600 dark:text-gray-400">{m.users_no_match()}</p>
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
</div>
