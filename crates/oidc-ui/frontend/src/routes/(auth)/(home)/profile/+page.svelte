<script lang="ts">
	import type { PageProps } from './$types';
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import UsernameForm from './UsernameForm.svelte';
	import PasswordForm from './PasswordForm.svelte';
	import InfoForm from './InfoForm.svelte';
	import { ArrowLeft } from '@lucide/svelte';
	import { resolve } from '$app/paths';

	let { data }: PageProps = $props();

	let user = $derived(data.user) as User;
</script>

<svelte:head>
	<title>Profile</title>
</svelte:head>

<div class="space-y-4">
	<section class="card">
		<div class="flex flex-row items-center justify-start space-x-2">
			<a href={resolve('/')}>
				<ArrowLeft />
			</a>
			<h2 class="m-0">Profile</h2>
		</div>
	</section>

	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<UsernameForm {user} />
		<PasswordForm {user} />
	</div>

	<InfoForm {user} />
</div>
