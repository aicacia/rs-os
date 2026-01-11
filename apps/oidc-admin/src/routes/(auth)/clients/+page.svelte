<script lang="ts">
	import type { PageProps } from './$types';
	import type { Client } from '$lib/common/openapi/oidc-admin/models/index';
	import { Plus, Pencil, Trash,  Search, ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { notifications } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data }: PageProps = $props();

	let searchQuery = $state('');
	let clients = $derived(data.clients);

	const filteredClients = $derived(
		clients.filter((client) => client.name.toLowerCase().includes(searchQuery.toLowerCase()))
	);

	async function deleteClient(client: Client) {
		if (!confirm(m.clients_delete_confirm({ name: client.name }))) {
			return;
		}

		try {
			await clientApi.clientDelete({ clientId: client.id });
			clients = clients.filter((c) => c.id !== client.id);
			notifications.add(m.clients_deleted_success(), 'success');
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

<section class="card">
	<div class="flex items-center justify-between gap-4">
		<div class="flex items-center gap-4">
			<a href={resolve('/')}>
				<ArrowLeft />
			</a>
			<h2>{m.clients_oauth2_clients()}</h2>
		</div>
		<a
			class="btn primary flex items-center gap-2"
			href={resolve('/(auth)/clients/create')}
		>
			<Plus class="h-5 w-5" />
			{m.clients_create_new()}
	</a>
	</div>

	<div class="relative">
		<Search class="absolute top-1/2 left-3 h-5 w-5 -translate-y-1/2" />
		<input
			type="text"
			placeholder={m.clients_search_placeholder()}
			bind:value={searchQuery}
			class="w-full pl-10"
		/>
	</div>
</section>

{#if filteredClients.length === 0}
	<section class="card mt-4">
		<p>{searchQuery ? m.clients_no_match() : m.clients_no_clients()}</p>
	</section>
{:else}
	<section class="card mt-4">
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead>
					<tr>
						<th class="px-4 py-3 text-left">{m.clients_name()}</th>
						<th class="px-4 py-3 text-left">{m.clients_client_id()}</th>
						<th class="px-4 py-3 text-left">{m.clients_type()}</th>
						<th class="px-4 py-3 text-left">{m.clients_status()}</th>
						<th class="px-4 py-3 text-right">{m.clients_actions()}</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredClients as client}
						<tr>
							<td class="px-4 py-3">{client.name}</td>
							<td class="px-4 py-3 font-mono">{client.clientId}</td>
							<td class="px-4 py-3">
								<span class="badge info">
									{client.applicationType}
								</span>
							</td>
							<td class="px-4 py-3">
								{#if client.active}
									<span class="badge success">
										{m.clients_active()}
									</span>
								{:else}
									<span class="badge danger">
										{m.clients_inactive()}
									</span>
								{/if}
							</td>
							<td class="px-4 py-3 text-right">
								<div class="flex items-center justify-end gap-2">
									<a
										class="btn icon primary sm"
										href={resolve('/(auth)/clients/[clientId=integer]/edit', {
											clientId: client.id.toString()
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
