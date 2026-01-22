import { EventEmitter } from 'eventemitter3';

export interface ServicesJSONResponse {
	oidc: string;
	fs_api: string;
	document_store_api: string;
	signaling_api: string;
}

type ServicesEvents = {
	'discovered-services': (services: ServicesJSONResponse) => void;
	error: (error: unknown) => void;
};

export class ServiceDiscovery extends EventEmitter<ServicesEvents> {
	private baseUrl: string;
	private current: ServicesJSONResponse | null = null;
	private currentPromise: Promise<ServicesJSONResponse>;
	private abortController: AbortController | null = null;

	constructor(baseUrl: string) {
		super();
		this.baseUrl = baseUrl;
		this.currentPromise = this.discoverServices();
	}

	private cancelOngoingRequest() {
		if (this.abortController) {
			this.abortController.abort();
			this.abortController = null;
		}
	}

	private isAbortError(error: unknown): boolean {
		return error instanceof Error && error.name === 'AbortError';
	}

	private async discoverServices(): Promise<ServicesJSONResponse> {
		this.cancelOngoingRequest();
		const abortController = new AbortController();
		this.abortController = abortController;

		try {
			const response = await fetch(`${this.baseUrl}/.well-known/services`, {
				method: 'GET',
				signal: abortController.signal
			});
			if (!response.ok) {
				throw new Error(
					`Failed to fetch services from ${this.baseUrl}/.well-known/services: ${response.statusText}`
				);
			}
			const services = (await response.json()) as ServicesJSONResponse;
			this.current = services;
			this.emit('discovered-services', services);
			return services;
		} catch (error) {
			if (!this.isAbortError(error)) {
				this.emit('error', error);
			}
			throw error;
		} finally {
			if (this.abortController === abortController) {
				this.abortController = null;
			}
		}
	}

	setBaseUrl(baseUrl: string) {
		if (this.baseUrl === baseUrl) {
			return this;
		}
		this.baseUrl = baseUrl;
		this.refresh();
		return this;
	}

	waitForServices(): Promise<ServicesJSONResponse> {
		if (this.current) {
			return Promise.resolve(this.current);
		}
		return this.currentPromise;
	}

	isReady(): boolean {
		return this.current !== null;
	}

	refresh(): Promise<ServicesJSONResponse> {
		this.currentPromise = this.discoverServices();
		return this.currentPromise;
	}
}
