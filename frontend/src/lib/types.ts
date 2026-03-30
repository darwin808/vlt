export type SecretType = 'password' | 'api_key' | 'env_var' | 'ssh_key' | 'note';

export interface SecretRecord {
	id: string;
	name: string;
	type: SecretType;
	encrypted_value: string;
	iv: string;
	url: string;
	username: string;
	created_at: string;
}

export interface BrowserPasswordEntry {
	name: string;
	url: string;
	username: string;
	password: string;
	note?: string;
}
