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
