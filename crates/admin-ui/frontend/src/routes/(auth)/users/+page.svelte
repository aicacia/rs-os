<script lang="ts">
	import type { PageProps } from './$types';
	import { Search, Plus, ArrowLeft, Eye, Pencil } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { hasPermissions } from '$lib/common/state/user.svelte';
	import { Permission, type User as OUser } from '$lib/common/openapi/admin/models';

	let { data }: PageProps = $props();
	let query = $state('');
	let users = $state<OUser[]>(data.users ?? []);
	let canCreate = $state(hasPermissions(data.user, [Permission.UserWrite]));

	$effect(() => {
		users = data.users ?? [];
	});

	$effect(() => {
		canCreate = hasPermissions(data.user, [Permission.UserWrite]);
	});

	const filtered = $derived.by(() => {
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
	<div class="flex gap-4 items-center justify-between">
		<div class="flex gap-4 items-center">
			<a href={resolve('/')}>
				<ArrowLeft />
			</a>
			<h2>{m.users_title()}</h2>
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

	<div class="relative">
		<Search class="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2" />
		<input
			type="text"
			placeholder={m.users_search_placeholder()}
			bind:value={query}
			class="w-full pl-10"
		/>
	</div>
</section>

<section class="card">
	{#if users.length === 0}
		<p>{m.users_empty()}</p>
	{:else if filtered.length === 0}
		<p>{m.users_no_match()}</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead>
					<tr>
						<th class="px-4 py-3 text-left">{m.users_username()}</th>
						<th class="px-4 py-3 text-left">{m.users_id()}</th>
						<th class="px-4 py-3 text-left">{m.users_created_at()}</th>
						<th class="px-4 py-3 text-left">{m.users_updated_at()}</th>
						<th class="px-4 py-3 text-right">{m.users_actions()}</th>
					</tr>
				</thead>
				<tbody>
					{#each filtered as u (u.id)}
						<tr>
							<td class="px-4 py-3">{u.username}</td>
							<td class="px-4 py-3">{u.id}</td>
							<td class="px-4 py-3">{new Date(u.createdAt).toLocaleString()}</td>
							<td class="px-4 py-3">{new Date(u.updatedAt).toLocaleString()}</td>
							<td class="px-4 py-3 text-right">
								<div class="flex items-center justify-end gap-2">
									<button
										class="btn icon light sm"
										onclick={() => onView(u)}
										aria-label="view user"
									>
										<Eye class="h-4 w-4" />
									</button>
									{#if canCreate}
										<button
											class="btn icon primary sm"
											onclick={() => onEdit(u)}
											aria-label={m.users_edit_title()}
										>
											<Pencil class="h-4 w-4" />
										</button>
									{/if}
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</section>
</div>