import { createNotification } from './state/notifications.svelte';
import * as m from '$lib/paraglide/messages';
import { ResponseError, type HttpError } from './openapi/oidc';

export type Errors = HttpError['messages'];

export async function getErrors(error: unknown) {
	if (error instanceof ResponseError) {
		const httpError = (await error.response.json()) as Errors;
		if (httpError) {
			return httpError;
		}
	} else if (error != null && typeof error === 'object' && 'messages' in error) {
		return error as Errors;
	}
	throw error;
}

export async function handleError(error: unknown) {
	try {
		notifyErrors(await getErrors(error));
	} catch (e) {
		console.error(e);
		createNotification(`${m.errors_message_application()}: ${m.errors_message_application()}`);
		throw e;
	}
}

export function notifyErrors(errors: Errors) {
	for (const [name, message] of translateErrors(errors)) {
		createNotification(`${name}: ${message}`);
	}
}

export function translateErrors(errors: Errors) {
	const translatedErrors: [name: string, message: string][] = [];
	for (const [nameKey, messages] of Object.entries(errors)) {
		for (const message of messages) {
			const errorsNameKey = `errors_name_${nameKey.replaceAll('-', '_')}` as keyof typeof m;
			const errorsMessageKey =
				`errors_message_${message.code.replaceAll('-', '_')}` as keyof typeof m;
			const nameFn = m[errorsNameKey];
			if (typeof nameFn !== 'function') {
				console.error('Unknown error name key:', errorsNameKey);
				continue;
			}
			const bodyFn = m[errorsMessageKey];
			if (typeof bodyFn !== 'function') {
				console.error('Unknown error message key:', errorsMessageKey);
				continue;
			}
			// @ts-expect-error - nameFn should not have parameters
			const name = nameFn();
			// @ts-expect-error - bodyFn parameters are dynamic
			const body = bodyFn(message.parameters);
			translatedErrors.push([name, body]);
		}
	}
	return translatedErrors;
}

export function createFormatedError(errors: [name: string, message: string][]) {
	return errors.map(([name, message]) => `${name}: ${message}`).join('\n');
}
