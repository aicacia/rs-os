<script lang="ts">
	import type { PageProps } from './$types';
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { UsernameForm, InfoForm, PasswordForm } from '$lib/common/components/UserProfile';
	import { userApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { notifications } from '$lib/common/state/notifications.svelte';

	let { data }: PageProps = $props();

	let user: any = $derived(data.user);

	async function updateUsername(username: string) {
		try {
			const id = typeof user?.user === 'number' ? user.user : (typeof user?.id === 'string' ? Number.parseInt(user.id) : undefined);
			if (!id) {
				throw new Error('Missing user id');
			}
			const updated = await userApi.updateUserHandler({ id, updateUserRequest: { username } });
			user.username = updated.username ?? username;
			notifications.add(m.users_updated_success(), 'success');
		} catch (e) {
			await handleError(e);
		}
	}

	async function updateUserInfo(info: any) {
		try {
			const id = typeof user?.user === 'number' ? user.user : (typeof user?.id === 'string' ? Number.parseInt(user.id) : undefined);
			if (!id) {
				throw new Error('Missing user id');
			}
			const updated = await userApi.updateUserHandler({ id, updateUserRequest: { info } });
			user.info = updated.info ?? info;
			notifications.add(m.users_updated_success(), 'success');
			return updated.info ?? info;
		} catch (e) {
			await handleError(e);
			throw e;
		}
	}

	async function updatePassword(password: string) {
		try {
			const id = typeof user?.user === 'number' ? user.user : (typeof user?.id === 'string' ? Number.parseInt(user.id) : undefined);
			if (!id) {
				throw new Error('Missing user id');
			}
			await userApi.updateUserHandler({ id, updateUserRequest: { password } });
			notifications.add(m.profile_password_changed_success(), 'success');
		} catch (e) {
			await handleError(e);
		}
	}
</script>

<svelte:head>
	<title>Profile</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex items-center gap-4">
			<a href={resolve('/')}>
				<ArrowLeft />
			</a>
			<h2>{m.profile_title()}</h2>
		</div>
	</section>

	<UsernameForm username={user.username} onUpdate={updateUsername} />

	<InfoForm userInfo={user.info ?? {}} onUpdate={updateUserInfo} />

	<PasswordForm onUpdate={updatePassword} />
</div>
