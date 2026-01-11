<script lang="ts">
	import type { PageProps } from './$types';
	import { Search, Plus, ArrowLeft, Eye, Pencil } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { hasPermissions } from '$lib/common/state/user.svelte';
	import { Permission, type User as OUser } from '$lib/common/openapi/oidc-admin/models';

	let { data }: PageProps = $props();
	let query = $state('');
	let users = $derived<OUser[]>(data.users ?? []);
	let canCreate = $derived(hasPermissions(data.user, [Permission.UserWrite]));

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
</script>

<svelte:head>
	<title>{m.users_title()}</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-4">
				<a href={resolve('/')}>
					<ArrowLeft />
				</a>
				<h2>{m.users_title()}</h2>
			</div>
			{#if canCreate}
				<a class="btn primary flex items-center gap-2" href={resolve('/(auth)/users/create')}>
					<Plus class="h-5 w-5" />
					{m.users_create_label()}
				</a>
			{/if}
		</div>

		<div class="relative">
			<Search class="absolute top-1/2 left-3 h-5 w-5 -translate-y-1/2" />
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
										<a
											class="btn icon light sm"
											href={resolve('/(auth)/users/[id=integer]', { id: u.id.toString() })}
											aria-label="view user"
										>
											<Eye class="h-4 w-4" />
									</a>
										{#if canCreate}
											<a
												class="btn icon primary sm"
												href={resolve('/(auth)/users/[id=integer]/edit', { id: u.id.toString() })}
												aria-label={m.users_edit_title()}
											>
												<Pencil class="h-4 w-4" />
								</a>
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
