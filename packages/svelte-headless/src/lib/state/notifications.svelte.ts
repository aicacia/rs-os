import { unsafeId } from '$lib/util/unsafeId';

export type DefaultNotificationType = 'success' | 'error' | 'info' | 'warning';

export interface Notification<T = DefaultNotificationType> {
	id: string;
	message: string;
	type: T;
}

export interface CreateNotificationsOptions {
	generateId?: () => string;
}

export function createNotifications<T = DefaultNotificationType>(
	options: CreateNotificationsOptions = {}
) {
	const { generateId = unsafeId } = options;
	const notifications = $state<Notification<T>[]>([]);

	function add(message: string, type: T, deleteAfterMS = 5000): string {
		const id = generateId();

		notifications.push({
			id,
			message,
			type
		});
		if (deleteAfterMS > 0) {
			setTimeout(() => remove(id), deleteAfterMS);
		}
		return id;
	}

	function remove(id: string): void {
		const index = notifications.findIndex((notification) => notification.id === id);

		if (index !== -1) {
			notifications.splice(index, 1);
		}
	}

	return {
		get items() {
			return notifications;
		},
		add,
		remove
	};
}
