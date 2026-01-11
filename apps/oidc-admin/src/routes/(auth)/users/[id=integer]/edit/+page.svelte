<script lang="ts">
	import type { PageProps } from './$types';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import UserForm from '../../create/UserForm.svelte';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import DeleteUserDialog from '../DeleteUserDialog.svelte';
	import { notifications } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data }: PageProps = $props();
	let user = data.user;
	let showDelete = $state(false);

	async function onSubmit(values: { username: string }) {
		try {
			const updated = await userApi.updateUserHandler({
				id: user.id,
				updateUserRequest: { username: values.username }
			});
			notifications.add(m.users_updated_success(), 'success');
			await goto(resolve(`/(auth)/users/[id=integer]`, { id: updated.id }));
		} catch (e) {
			await handleError(e);
		}
	}

	async function onDeleteConfirm() {
		try {
			await userApi.deleteUserHandler({ id: user.id });
			notifications.add(m.users_deleted_success(), 'success');
			await goto(resolve('/(auth)/users'));
		} catch (e) {
			await handleError(e);
		}
	}
</script>

<svelte:head>
	<title>{m.users_edit_title()} - {user.username}</title>
</svelte:head>

<section class="card">
	<UserForm
		mode="edit"
		initial={{ username: user.username }}
		{onSubmit}
		onCancel={() => history.back()}
	/>
	{#if showDelete}
		<DeleteUserDialog
			username={user.username}
			message={m.users_delete_confirm({ username: user.username })}
			onConfirm={onDeleteConfirm}
			onCancel={() => (showDelete = false)}
			open
		/>
	{/if}
	<div>
		<button class="btn danger" onclick={() => (showDelete = true)}>
			{m.actions_delete()}
		</button>
	</div>
</section>
