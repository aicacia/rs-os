<script lang="ts">
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import ClientForm, { type ClientFormData } from '../_ClientForm.svelte';
	import { m } from '$lib/paraglide/messages';

	async function handleSubmit(data: ClientFormData) {
		try {
			const client = await clientApi.clientCreate({
				clientRegisterRequest: data
			});
			createNotification(m.clients_created_success(), 'success');
			await goto(resolve('/(auth)/clients'));
		} catch (e) {
			handleError(e);
		}
	}
</script>

<svelte:head>
	<title>{m.clients_create_title()}</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex items-center gap-3">
			<a
				href={resolve('/(auth)/clients')}
				class="text-gray-600 hover:text-gray-900 dark:hover:text-white"
			>
				<ArrowLeft class="h-5 w-5" />
			</a>
			<h2 class="m-0">{m.clients_create_title()}</h2>
		</div>
	</section>

	<ClientForm initialValues={{}} onsubmit={handleSubmit}>
		{#snippet actions()}
			<a type="button" class="btn secondary" href={resolve('/(auth)/clients')}>
				{m.clients_cancel()}
			</a>
		{/snippet}
	</ClientForm>
</div>
