<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const UsernameFormSchema = () =>
		v.object({
			username: v.pipe(v.string(), v.minLength(1))
		});
</script>

<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';

	let { user = $bindable() }: { user: User } = $props();

	const form = createForm(UsernameFormSchema(), {
		username: user.username ?? ''
	});

	$effect(() => {
		form.fields.username.value = user.username ?? '';
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			user = await currentUserApi.updateUsername({ updateUsernameRequest: value });
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
			bind:value={form.fields.username.value}
			placeholder={m.profile_username_placeholder()}
			aria-label={m.profile_username_label()}
		/>
		<Issues issues={form.fields.username.issues} />
	</label>
	<button type="submit" class="btn primary mt-4">{m.profile_save_username()}</button>
</form>
