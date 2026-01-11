<script lang="ts">
	import { goto } from '$app/navigation';
	import UserForm from './UserForm.svelte';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { notifications } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';
	import { resolve } from '$app/paths';

	async function onSubmit(values: { username: string }) {
		try {
			const user = await userApi.createUserHandler({
				createUserRequest: { username: values.username }
			});
			notifications.add(m.users_created_success(), 'success');
			await goto(resolve(`/(auth)/users/[id=integer]`, { id: user.id }));
		} catch (e) {
			await handleError(e);
		}
	}
</script>

<svelte:head>
	<title>{m.users_create_title()}</title>
</svelte:head>

<section class="card">
	<UserForm mode="create" {onSubmit} onCancel={() => history.back()} />
</section>
