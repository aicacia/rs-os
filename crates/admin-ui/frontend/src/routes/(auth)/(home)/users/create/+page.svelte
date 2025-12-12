<script lang="ts">
	import { goto } from '$app/navigation';
	import UserForm from './UserForm.svelte';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createNotification } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';

	async function onSubmit(values: { username: string }) {
		try {
			const user = await userApi.createUserHandler({
				createUserRequest: { username: values.username }
			});
			createNotification(m.users_created_success(), 'success');
			goto(`/users/${user.id}`);
		} catch (e) {
			await handleError(e);
		}
	}
</script>

<svelte:head>
	<title>{m.users_create_title()}</title>
</svelte:head>

<section class="mx-auto max-w-xl p-4">
	<UserForm mode="create" {onSubmit} onCancel={() => history.back()} />
</section>
