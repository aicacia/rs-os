import { createNotification } from './state/notifications.svelte';
import * as m from '$lib/paraglide/messages';
import { ResponseError, type HttpError } from './openapi/oidc';

export type Errors = HttpError['messages'];

export async function handleError(error: unknown) {
	if (error instanceof ResponseError) {
		const httpError = (await error.response.json()) as Errors;
		if (httpError) {
			notifyErrors(httpError);
			return httpError;
		}
	} else if (error != null && typeof error === 'object' && 'messages' in error) {
		const errors = error as Errors;
		notifyErrors(errors);
		return errors as Errors;
	}
	console.error(error);
	createNotification(`${m.errors_message_application()}: ${m.errors_message_application()}`);
	throw error;
}

export function notifyErrors(errors: Errors) {
	for (const [name, message] of getErrors(errors)) {
		createNotification(`${name}: ${message}`);
	}
}

export function getErrors(errors: Errors) {
	const translatedErrors: [name: string, message: string][] = [];
	for (const [nameKey, messages] of Object.entries(errors)) {
		for (const message of messages) {
			const errorsNameKey = `errors_name_${nameKey.replaceAll('-', '_')}`;
			const errorsMessageKey = `errors_message_${message.code.replaceAll('-', '_')}`;
			// @ts-expect-error not gonna be type safe
			const name = m[errorsNameKey]();
			// @ts-expect-error not gonna be type safe
			const body = m[errorsMessageKey](message.parameters);
			translatedErrors.push([name, body]);
		}
	}
	return translatedErrors;
}

export function createFormatedError(errors: [name: string, message: string][]) {
	return errors.map(([name, message]) => `${name}: ${message}`).join('\n');
}
