<script lang="ts">
	import type { PageProps } from './$types';
	import type { CurrentUser } from '$lib/common/openapi/admin';
	import UsernameForm from './UsernameForm.svelte';
	import PasswordForm from './PasswordForm.svelte';
	import InfoForm from './InfoForm.svelte';
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';

	import { m } from '$lib/paraglide/messages';

	let { data }: PageProps = $props();

	let user = $derived(data.user) as CurrentUser;
</script>

<svelte:head>
	<title>Profile</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex gap-4 flex-row items-center justify-between">
			<div class="flex gap-4 flex-row items-center">
				<a href={resolve('/')}>
					<ArrowLeft />
				</a>
				<h2 class="m-0">{m.profile_title()}</h2>
			</div>
		</div>
	</section>

	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<UsernameForm {user} />
		<PasswordForm {user} />
	</div>

	<InfoForm {user} />
</div>
