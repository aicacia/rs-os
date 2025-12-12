import { clientApi } from '$lib/common/openapi';
import type { AuthorizeRequest, Client } from '$lib/common/openapi/oidc';
import {
	ClientRegisterRequestFromJSON,
	type ClientRegisterRequest
} from '$lib/common/openapi/oidc/models/ClientRegisterRequest';

export type ClientInfo = ClientRegisterRequest;
export const ClientInfoFromJSON = ClientRegisterRequestFromJSON;

export function getClientDiff(client: Client, clientInfo: ClientInfo): Partial<ClientInfo> | false {
	const diff: Partial<ClientInfo> = {};
	let changed = false;

	for (const key of Object.keys(clientInfo) as (keyof ClientInfo)[]) {
		const a = client[key];
		const b = clientInfo[key];

		const aIsArray = Array.isArray(a);
		const bIsArray = Array.isArray(b);

		if (aIsArray !== bIsArray) {
			changed = true;
			diff[key] = a as never;
			continue;
		}

		if (aIsArray && bIsArray) {
			const arrA = a as unknown as string[];
			const arrB = b as unknown as string[];

			const arrChanged = arrA.length !== arrB.length || arrA.some((v, i) => v !== arrB[i]);

			if (arrChanged) {
				changed = true;
				diff[key] = a as never;
			}

			continue;
		}

		if (a !== b) {
			changed = true;
			diff[key] = a as never;
		}
	}

	if (!changed) {
		return false;
	}

	return diff;
}

export function rejectAuthorizeRequest(
	authorizeRequest: Pick<AuthorizeRequest, 'redirectUri' | 'state' | 'nonce'>,
	error: string,
	errorDescription: string
) {
	const url = new URL(authorizeRequest.redirectUri);
	if (authorizeRequest.state) {
		url.searchParams.append('state', authorizeRequest.state);
	}
	if (authorizeRequest.nonce) {
		url.searchParams.append('nonce', authorizeRequest.nonce);
	}
	url.searchParams.append('error', error);
	url.searchParams.append('error_description', errorDescription);
	window.location.href = url.toString();
}

export async function resolveAuthorizeRequest(authorizeRequest: AuthorizeRequest) {
	const url = new URL(authorizeRequest.redirectUri);
	if (authorizeRequest.state) {
		url.searchParams.append('state', authorizeRequest.state);
	}
	if (authorizeRequest.nonce) {
		url.searchParams.append('nonce', authorizeRequest.nonce);
	}
	const authorizeResponse = await clientApi.clientAuthorize({
		clientId: authorizeRequest.clientId,
		clientAuthorizeRequest: authorizeRequest
	});
	switch (authorizeRequest.responseMode) {
		case 'fragment':
		case 'query': {
			switch (authorizeResponse.type) {
				case 'authorization_code': {
					url.searchParams.set('code', authorizeResponse.code);
					break;
				}
				case 'implicit':
				case 'hybrid': {
					url.searchParams.set('access_token', authorizeResponse.accessToken);
					url.searchParams.set('token_type', authorizeResponse.tokenType);
					url.searchParams.set('expires_in', authorizeResponse.expiresIn);
					if (authorizeResponse.idToken) {
						url.searchParams.set('id_token', authorizeResponse.idToken);
					}
					break;
				}
			}
			window.location.href = url.toString();
			break;
		}
		case 'form_post': {
			throw new Error('not supported yet!');
			break;
		}
		case 'web_message': {
			throw new Error('not supported yet!');
			break;
		}
	}
}
