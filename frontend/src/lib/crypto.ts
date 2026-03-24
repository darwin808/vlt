const PBKDF2_ITERATIONS = 600_000;
const KEY_LENGTH = 256;
const IV_LENGTH = 12;
const SALT_LENGTH = 16;

function toBase64(buf: ArrayBuffer): string {
	return btoa(String.fromCharCode(...new Uint8Array(buf)));
}

function fromBase64(b64: string): Uint8Array {
	return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

export function generateSalt(): string {
	const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
	return toBase64(salt);
}

export async function deriveKey(password: string, saltB64: string): Promise<CryptoKey> {
	const encoder = new TextEncoder();
	const salt = fromBase64(saltB64);

	const keyMaterial = await crypto.subtle.importKey('raw', encoder.encode(password), 'PBKDF2', false, [
		'deriveKey'
	]);

	return crypto.subtle.deriveKey(
		{ name: 'PBKDF2', salt, iterations: PBKDF2_ITERATIONS, hash: 'SHA-256' },
		keyMaterial,
		{ name: 'AES-GCM', length: KEY_LENGTH },
		false,
		['encrypt', 'decrypt']
	);
}

export async function encrypt(
	key: CryptoKey,
	plaintext: string
): Promise<{ ciphertext: string; iv: string }> {
	const encoder = new TextEncoder();
	const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));

	const encrypted = await crypto.subtle.encrypt(
		{ name: 'AES-GCM', iv },
		key,
		encoder.encode(plaintext)
	);

	return {
		ciphertext: toBase64(encrypted),
		iv: toBase64(iv)
	};
}

export async function decrypt(key: CryptoKey, ciphertextB64: string, ivB64: string): Promise<string> {
	const ciphertext = fromBase64(ciphertextB64);
	const iv = fromBase64(ivB64);

	const decrypted = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, ciphertext);

	return new TextDecoder().decode(decrypted);
}
