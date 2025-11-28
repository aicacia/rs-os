<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models';
	import { Edit, Trash2, User as UserIcon } from '@lucide/svelte';

	interface Props {
		user: User;
		onEdit?: (user: User) => void;
		onDelete?: (user: User) => void;
	}
	let { user, onEdit, onDelete }: Props = $props();
</script>

<div class="card secondary flex items-center justify-between gap-3">
	<div class="flex items-center gap-3">
		<UserIcon class="h-6 w-6" />
		<div>
			<div class="font-medium">{user.username}</div>
			<div class="text-xs text-gray-600 dark:text-gray-400">ID {user.id}</div>
		</div>
	</div>
	<div class="flex gap-2">
		{#if onEdit}
			<button class="btn light" onclick={() => onEdit?.(user)} aria-label="edit">
				<Edit class="h-4 w-4" />
			</button>
		{/if}
		{#if onDelete}
			<button class="btn danger" onclick={() => onDelete?.(user)} aria-label="delete">
				<Trash2 class="h-4 w-4" />
			</button>
		{/if}
	</div>
</div>
