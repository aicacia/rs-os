<script lang="ts">
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { clientApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import ClientForm, { type ClientFormData } from '../_ClientForm.svelte';

	async function handleSubmit(data: ClientFormData) {
		try {
			const client = await clientApi.clientCreate({
				clientRegisterRequest: data
			});
			createNotification('Client created successfully', 'success');
			await goto(resolve('/(auth)/(home)/clients'));
		} catch (e) {
			handleError(e);
		}
	}
</script>

<svelte:head>
	<title>Create Client</title>
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
			<h2 class="m-0 text-2xl font-bold">Create New Client</h2>
		</div>
	</section>

	<ClientForm initialValues={{}} onsubmit={handleSubmit} submitLabel="Create Client">
		{#snippet actions()}
			<a type="button" class="btn secondary" href={resolve('/(auth)/(home)/clients')}> Cancel </a>
		{/snippet}
	</ClientForm>
</div>
