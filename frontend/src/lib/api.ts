const BASE = '';  // same origin in production, Vite proxy in dev

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(BASE + path, {
		credentials: 'include',
		headers: { 'Content-Type': 'application/json', ...options?.headers },
		...options
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({}));
		throw new Error(body.error || `Request failed: ${res.status}`);
	}
	return res.json();
}

export const api = {
	auth: {
		signup: (data: { email: string; password: string; encryption_salt: string }) =>
			request<{ id: string; email: string; encryption_salt: string }>('/api/auth/signup', {
				method: 'POST',
				body: JSON.stringify(data)
			}),
		login: (data: { email: string; password: string }) =>
			request<{ id: string; email: string; encryption_salt: string }>('/api/auth/login', {
				method: 'POST',
				body: JSON.stringify(data)
			}),
		logout: () => request('/api/auth/logout', { method: 'POST' }),
		me: () => request<{ id: string; email: string; encryption_salt: string }>('/api/auth/me')
	},
	secrets: {
		list: () =>
			request<
				Array<{
					id: string;
					name: string;
					type: string;
					encrypted_value: string;
					iv: string;
					username: string;
					url: string;
					created_at: string;
				}>
			>('/api/secrets'),
		create: (data: {
			name: string;
			type: string;
			encrypted_value: string;
			iv: string;
			username?: string;
			url?: string;
		}) => request('/api/secrets', { method: 'POST', body: JSON.stringify(data) }),
		delete: (id: string) => request(`/api/secrets/${id}`, { method: 'DELETE' }),
		touch: (id: string) => request(`/api/secrets/${id}/used`, { method: 'PATCH' })
	}
};
