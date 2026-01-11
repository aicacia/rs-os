<script lang="ts">
	import type { PageProps } from './$types';
	import { Pencil, Trash2 } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import DeleteUserDialog from './DeleteUserDialog.svelte';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { notifications } from '$lib/common/state/notifications.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { User } from '$lib/common/openapi/oidc-admin/models';
	import { resolve } from '$app/paths';

	let { data }: PageProps = $props();

	let user: User = $derived(data.user);
	let showDelete = $state(false);

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
	<title>{m.users_detail_title()} - {user.username}</title>
</svelte:head>

<section class="card">
	<h1>{m.users_detail_title()}</h1>

	<div>
		<span>ID</span>
		<div class="font-mono">{user.id}</div>
	</div>
	<div>
		<span>Username</span>
		<div>{user.username}</div>
	</div>
	<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
		<div>
			<span>Created</span>
			<div>{new Date(user.createdAt).toLocaleString()}</div>
		</div>
		<div>
			<span>Updated</span>
			<div>{new Date(user.updatedAt).toLocaleString()}</div>
		</div>
	</div>
	<div class="flex flex-row items-center gap-2">
		<a
			class="btn light flex flex-row items-center"
			href={resolve('/(auth)/users/[id=integer]/edit', { id: user.id })}
		>
			<Pencil class="mr-1 h-4 w-4" />
			{m.actions_edit()}
		</a>
		<button class="btn danger flex flex-row items-center" onclick={() => (showDelete = true)}>
			<Trash2 class="mr-1 h-4 w-4" />
			{m.actions_delete()}
		</button>
	</div>

	{#if showDelete}
		<DeleteUserDialog
			username={user.username}
			message={m.users_delete_confirm({ username: user.username })}
			onConfirm={onDeleteConfirm}
			onCancel={() => (showDelete = false)}
			open
		/>
	{/if}
</section>
