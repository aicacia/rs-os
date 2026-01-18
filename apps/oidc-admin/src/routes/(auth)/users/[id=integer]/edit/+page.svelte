<script lang="ts">
	import type { PageProps } from './$types';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { UsernameForm, InfoForm, PasswordForm } from '$lib/common/components/UserProfile';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import DeleteUserDialog from '../DeleteUserDialog.svelte';
	import { notifications } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';
	import { ArrowLeft } from '@lucide/svelte';

	let { data }: PageProps = $props();
	let user = $derived(data.user);
	let showDelete = $state(false);

	async function updateUsername(username: string) {
		try {
			const updated = await userApi.updateUserHandler({
				id: parseInt(user.id),
				updateUserRequest: { username }
			});
			user = updated;
			notifications.add(m.users_updated_success(), 'success');
		} catch (e) {
			await handleError(e);
		}
	}

	async function updateUserInfo(info: any) {
		try {
			const updated = await userApi.updateUserHandler({
				id: parseInt(user.id),
				updateUserRequest: { info }
			});
			user = updated;
			notifications.add(m.users_updated_success(), 'success');
			return updated.info ?? info;
		} catch (e) {
			await handleError(e);
			throw e;
		}
	}

	async function updatePassword(password: string) {
		try {
			await userApi.updateUserHandler({
				id: parseInt(user.id),
				updateUserRequest: { password }
			});
			notifications.add(m.profile_password_changed_success(), 'success');
		} catch (e) {
			await handleError(e);
		}
	}

	async function onDeleteConfirm() {
		try {
			await userApi.deleteUserHandler({ id: parseInt(user.id) });
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

<div class="space-y-4">
	<section class="card">
		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-4">
				<a href={resolve(`/(auth)/users`)}>
					<ArrowLeft />
				</a>
				<h2>{m.users_edit_title()}: {user.username}</h2>
			</div>
			<button class="btn danger" onclick={() => (showDelete = true)}>
				{m.actions_delete()}
			</button>
		</div>
	</section>

	<UsernameForm username={user.username} onUpdate={updateUsername} />

	<InfoForm userInfo={user.info ?? {}} onUpdate={updateUserInfo} />

	<PasswordForm onUpdate={updatePassword} />

	{#if showDelete}
		<DeleteUserDialog
			username={user.username}
			message={m.users_delete_confirm({ username: user.username })}
			onConfirm={onDeleteConfirm}
			onCancel={() => (showDelete = false)}
			open
		/>
	{/if}
</div>

