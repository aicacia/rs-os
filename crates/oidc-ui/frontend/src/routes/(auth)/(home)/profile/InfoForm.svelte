<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { UpdateUserInfoRequest, User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';

	let { user = $bindable() }: { user: User } = $props();

	let infoForm = $state<UpdateUserInfoRequest>({
		name: undefined,
		givenName: undefined,
		familyName: undefined,
		website: undefined,
		locale: undefined
	});

	$effect(() => {
		infoForm = {
			name: user.info?.name ?? user.username ?? '',
			givenName: user.info?.givenName ?? undefined,
			familyName: user.info?.familyName ?? undefined,
			website: user.info?.website ?? undefined,
			locale: user.info?.locale ?? undefined
		};
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		try {
			user = await currentUserApi.updateUserInfo({ updateUserInfoRequest: infoForm });
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3 class="text-lg font-medium">Profile Info</h3>
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<label class="block">
			<span class="text-sm font-medium">Full name</span>
			<input class="mt-1 block w-full px-3 py-2" bind:value={infoForm.name} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">Given name</span>
			<input class="mt-1 block w-full px-3 py-2" bind:value={infoForm.givenName} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">Family name</span>
			<input class="mt-1 block w-full px-3 py-2" bind:value={infoForm.familyName} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">Website</span>
			<input class="mt-1 block w-full px-3 py-2" bind:value={infoForm.website} />
		</label>
		<label class="block md:col-span-2">
			<span class="text-sm font-medium">Locale</span>
			<input class="mt-1 block w-full px-3 py-2" bind:value={infoForm.locale} />
		</label>
	</div>
	<button type="submit" class="btn success mt-4">Save profile</button>
</form>
