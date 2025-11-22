<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { m } from '$lib/paraglide/messages';

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
	<h3 class="text-lg font-medium">{m.profile_username_title()}</h3>
	<label class="block">
		<span class="text-sm font-medium">{m.profile_username_label()}</span>
		<input
			id="profile-username"
			class="mt-1 block w-full px-3 py-2"
			bind:value={username}
			required
			placeholder={m.profile_username_placeholder()}
			aria-label={m.profile_username_label()}
			aria-required="true"
		/>
	</label>
	<button type="submit" class="btn primary mt-4">{m.profile_save_username()}</button>
</form>
