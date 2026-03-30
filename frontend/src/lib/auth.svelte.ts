import { api } from './api';
import { deriveKey, generateSalt, storeKey, restoreKey, clearStoredKey } from './crypto';

interface User {
	id: string;
	email: string;
	encryption_salt: string;
}

let user = $state<User | null>(null);
let encryptionKey = $state<CryptoKey | null>(null);
let ready = $state(false);

async function init() {
	try {
		const me = await api.auth.me();
		user = me;
		encryptionKey = await restoreKey();
	} catch {
		user = null;
		clearStoredKey();
	}
	ready = true;
}

init();

export const auth = {
	get user() {
		return user;
	},
	get isAuthenticated() {
		return user !== null;
	},
	get isUnlocked() {
		return encryptionKey !== null;
	},
	get isReady() {
		return ready;
	},
	get key() {
		return encryptionKey;
	},

	async login(email: string, password: string) {
		const record = await api.auth.login({ email, password });
		user = record;
		const salt = record.encryption_salt;
		encryptionKey = await deriveKey(password, salt);
		await storeKey(encryptionKey);
	},

	async signup(email: string, password: string) {
		const salt = generateSalt();
		await api.auth.signup({ email, password, encryption_salt: salt });
		await this.login(email, password);
	},

	async logout() {
		try {
			await api.auth.logout();
		} catch {
			// Ignore logout errors — clear local state regardless
		}
		encryptionKey = null;
		user = null;
		clearStoredKey();
	}
};
