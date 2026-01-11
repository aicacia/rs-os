export interface ServicesJSONResponse {
	oidc: string;
	fs: string;
	signaling: string;
}

export type ServicesPromise = Promise<ServicesJSONResponse> & {
	refresh: () => ServicesPromise;
};

export function createServices(baseUrl: string): ServicesPromise {
	const url = `${baseUrl}/.well-known/services`;

	async function discoverServices() {
		const response = await fetch(url, { method: 'GET' });
		if (!response.ok) {
			throw new Error(`Failed to fetch services from ${url}: ${response.statusText}`);
		}
		return (await response.json()) as ServicesJSONResponse;
	}

	let currentPromise = discoverServices();

	return {
		then(onFulfilled, onRejected) {
			return currentPromise.then(onFulfilled, onRejected);
		},
		catch(onRejected) {
			return currentPromise.catch(onRejected);
		},
		finally(onFinally) {
			return currentPromise.finally(onFinally);
		},
		refresh() {
			currentPromise = discoverServices();
			return this;
		},
		[Symbol.toStringTag]: 'Promise'
	};
}
