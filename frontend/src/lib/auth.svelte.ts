import pb from './pb';
import { deriveKey, generateSalt } from './crypto';
import type { RecordModel } from 'pocketbase';

let user = $state<RecordModel | null>(pb.authStore.record);
let encryptionKey = $state<CryptoKey | null>(null);

pb.authStore.onChange((_, record) => {
	user = record;
});

export const auth = {
	get user() {
		return user;
	},
	get isAuthenticated() {
		return pb.authStore.isValid;
	},
	get isUnlocked() {
		return encryptionKey !== null;
	},
	get key() {
		return encryptionKey;
	},

	async login(email: string, password: string) {
		const record = await pb.collection('users').authWithPassword(email, password);
		const salt = record.record.encryption_salt;
		encryptionKey = await deriveKey(password, salt);
	},

	async signup(email: string, password: string) {
		const salt = generateSalt();
		await pb.collection('users').create({
			email,
			password,
			passwordConfirm: password,
			encryption_salt: salt
		});
		await this.login(email, password);
	},

	async unlock(password: string) {
		if (!user) throw new Error('Not authenticated');
		const salt = user.encryption_salt as string;
		encryptionKey = await deriveKey(password, salt);
	},

	logout() {
		pb.authStore.clear();
		encryptionKey = null;
		user = null;
	}
};
