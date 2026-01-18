import { EventEmitter } from 'eventemitter3';

export interface ServicesJSONResponse {
	oidc: string;
	fs: string;
	signaling: string;
}

type ServicesEvents = {
	'discovered-services': (services: ServicesJSONResponse) => void;
	error: (error: unknown) => void;
};

export class ServiceDiscovery extends EventEmitter<ServicesEvents> {
	private baseUrl: string;
	private current: ServicesJSONResponse | null = null;
	private currentPromise: Promise<ServicesJSONResponse>;

	constructor(baseUrl: string) {
		super();
		this.baseUrl = baseUrl;
		this.currentPromise = this.discoverServices();
	}

	private async discoverServices(): Promise<ServicesJSONResponse> {
		try {
			const response = await fetch(`${this.baseUrl}/.well-known/services`, { method: 'GET' });
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
			this.emit('error', error);
			throw error;
		}
	}

	setBaseUrl(baseUrl: string) {
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
