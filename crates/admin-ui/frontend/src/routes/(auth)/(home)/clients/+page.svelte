<script lang="ts">
	import type { PageProps } from './$types';
	import type { Client } from '$lib/common/openapi/admin/models/index';
	import { Plus, Pencil, Trash, Eye, Search } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data }: PageProps = $props();

	let searchQuery = $state('');
	let clients = $state(data.clients);

	const filteredClients = $derived(
		clients.filter(
			(client) =>
				client.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				client.clientId.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	async function deleteClient(client: Client) {
		if (!confirm(m.clients_delete_confirm({ name: client.name }))) {
			return;
		}

		try {
			await clientApi.clientDelete({ clientId: client.clientId });
			clients = clients.filter((c) => c.clientId !== client.clientId);
			createNotification(m.clients_deleted_success(), 'success');
		} catch (e) {
			handleError(e);
		}
	}

	$effect(() => {
		clients = data.clients;
	});
</script>

<svelte:head>
	<title>{m.clients_title()}</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
			<h2 class="m-0 text-2xl font-bold">{m.clients_oauth2_clients()}</h2>
			<button
				class="btn primary flex items-center gap-2"
				onclick={() => goto(resolve('/clients/create'))}
			>
				<Plus class="h-5 w-5" />
				{m.clients_create_new()}
			</button>
		</div>

		<div class="mt-4">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400" />
				<input
					type="text"
					placeholder={m.clients_search_placeholder()}
					bind:value={searchQuery}
					class="w-full rounded-lg border border-gray-300 py-2 pl-10 pr-4 dark:border-gray-600 dark:bg-gray-800"
				/>
			</div>
		</div>
	</section>
	{#if filteredClients.length === 0}
		<section class="card">
			<div class="py-12 text-center">
				<p class="text-gray-500">
					{searchQuery ? m.clients_no_match() : m.clients_no_clients()}
				</p>
			</div>
		</section>
	{:else}
		<section class="card">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="border-b border-gray-200 dark:border-gray-700">
						<tr>
							<th class="px-4 py-3 text-left text-sm font-semibold">{m.clients_name()}</th>
							<th class="px-4 py-3 text-left text-sm font-semibold">{m.clients_client_id()}</th>
							<th class="px-4 py-3 text-left text-sm font-semibold">{m.clients_type()}</th>
							<th class="px-4 py-3 text-left text-sm font-semibold">{m.clients_status()}</th>
							<th class="px-4 py-3 text-right text-sm font-semibold">{m.clients_actions()}</th>
						</tr>
					</thead>
					<tbody
						class="border-b border-gray-100 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-800"
					>
						{#each filteredClients as client}
							<tr>
								<td class="px-4 py-3 font-medium">{client.name}</td>
								<td class="px-4 py-3 font-mono text-sm text-gray-600 dark:text-gray-400">
									{client.clientId}
								</td>
								<td class="px-4 py-3 text-sm">
									<span
										class="rounded-full bg-blue-100 px-2 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900 dark:text-blue-200"
									>
										{client.applicationType}
									</span>
								</td>
								<td class="px-4 py-3 text-sm">
									{#if client.active}
										<span
											class="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-200"
										>
											{m.clients_active()}
										</span>
									{:else}
										<span
											class="rounded-full bg-red-100 px-2 py-1 text-xs font-medium text-red-800 dark:bg-red-900 dark:text-red-200"
										>
											{m.clients_inactive()}
										</span>
									{/if}
								</td>
								<td class="px-4 py-3 text-right">
									<div class="flex items-center justify-end gap-2">
										<a
											class="btn icon primary sm"
											href={resolve('/(auth)/(home)/clients/[clientId]/edit', {
												clientId: encodeURIComponent(client.clientId)
											})}
											aria-label={m.clients_edit()}
										>
											<Pencil class="h-4 w-4" />
										</a>
										<button
											class="btn icon danger sm"
											onclick={() => deleteClient(client)}
											aria-label={m.clients_delete()}
										>
											<Trash class="h-4 w-4" />
										</button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>
	{/if}
</div>
