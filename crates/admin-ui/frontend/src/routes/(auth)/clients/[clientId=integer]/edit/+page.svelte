<script lang="ts">
	import type { PageProps } from './$types';
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import ClientForm, { type ClientFormData } from '../../_ClientForm.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data }: PageProps = $props();

	const client = $derived(data.client);

	async function handleSubmit(formData: ClientFormData) {
		try {
			const updatedClient = await clientApi.clientUpdate({
				clientId: client.id,
				clientUpsertRequest: formData
			});
			createNotification(m.clients_updated_success(), 'success');
			await goto(resolve(`/clients/${updatedClient.id}`));
		} catch (e) {
			handleError(e);
		}
	}
</script>

<svelte:head>
	<title>{m.clients_edit_title()}</title>
</svelte:head>

<section class="card">
	<div class="flex items-center gap-3">
		<a href={resolve('/(auth)/clients')}>
			<ArrowLeft class="h-5 w-5" />
		</a>
		<div>
			<h2>{m.clients_edit_title()}</h2>
			<p>{client.name}</p>
		</div>
	</div>
</section>

<ClientForm
	initialValues={client}
	onsubmit={handleSubmit}
>
	{#snippet actions()}
		<button
			type="button"
			class="btn secondary"
			onclick={() => goto(resolve('/(auth)/clients'))}
		>
			{m.clients_cancel()}
		</button>
	{/snippet}
</ClientForm>
