<script lang="ts">
	import type { PageProps } from './$types';
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import ClientForm, { type ClientFormData } from '../../_ClientForm.svelte';

	let { data }: PageProps = $props();

	const client = $derived(data.client);

	async function handleSubmit(formData: ClientFormData) {
		try {
			const updatedClient = await clientApi.clientUpdate({
				clientId: client.clientId,
				clientRegisterRequest: formData
			});
			createNotification('Client updated successfully', 'success');
			await goto(resolve(`/clients/${updatedClient.clientId}`));
		} catch (e) {
			handleError(e);
		}
	}
</script>

<svelte:head>
	<title>Edit {client.name}</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex items-center gap-3">
			<a
				href={resolve('/(auth)/(home)/clients')}
				class="text-gray-600 hover:text-gray-900 dark:hover:text-white"
			>
				<ArrowLeft class="h-5 w-5" />
			</a>
			<div>
				<h2 class="m-0 text-2xl font-bold">Edit Client</h2>
				<p class="text-sm text-gray-600 dark:text-gray-400">{client.name}</p>
			</div>
		</div>
	</section>

	<ClientForm initialValues={client} onsubmit={handleSubmit} submitLabel="Update Client">
		{#snippet actions()}
			<button
				type="button"
				class="btn secondary"
				onclick={() => goto(resolve('/(auth)/(home)/clients'))}
			>
				Cancel
			</button>
		{/snippet}
	</ClientForm>
</div>
