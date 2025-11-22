<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';

	let { user = $bindable() }: { user: User } = $props();

	let username = $state('');

	$effect(() => {
		username = user?.username ?? '';
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		try {
			user = await currentUserApi.updateUsername({ updateUsernameRequest: { username } });
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3 class="text-lg font-medium">Update Username</h3>
	<label class="block">
		<span class="text-sm font-medium">Username</span>
		<input class="mt-1 block w-full px-3 py-2" bind:value={username} required />
	</label>
	<button type="submit" class="btn primary mt-4">Save username</button>
</form>
