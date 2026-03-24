import type { RecordModel } from 'pocketbase';

export type SecretType = 'password' | 'api_key' | 'env_var' | 'ssh_key' | 'note';

export interface SecretRecord extends RecordModel {
	name: string;
	type: SecretType;
	encrypted_value: string;
	iv: string;
	url: string;
	username: string;
	user: string;
}

export interface BrowserPasswordEntry {
	name: string;
	url: string;
	username: string;
	password: string;
	note?: string;
}
