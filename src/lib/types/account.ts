/** A Minecraft account authenticated via Microsoft */
export interface MinecraftAccount {
	/** Unique identifier for this account entry */
	id: string;
	/** Minecraft username (display name) */
	username: string;
	/** Minecraft player UUID */
	uuid: string;
	/** Whether this is the currently active account */
	isActive: boolean;
	/** URL to player's skin texture */
	skinUrl?: string;
	/** URL to player's cape texture */
	capeUrl?: string;
	/** Unix timestamp when account was added */
	createdAt: number;
	/** Unix timestamp when account was last used */
	lastUsedAt: number;
	/** Unix timestamp when tokens expire */
	tokenExpiresAt: number;
}

/** Response from Microsoft device code flow initiation */
export interface DeviceCodeResponse {
	deviceCode: string;
	userCode: string;
	verificationUri: string;
	expiresIn: number;
	interval: number;
}

/** Status of device code authentication polling */
export type AuthPollStatus =
	| { status: 'pending' }
	| { status: 'success'; account: MinecraftAccount }
	| { status: 'expired' }
	| { status: 'error'; message: string };

/** Current authentication state */
export interface AuthState {
	/** Whether authentication is in progress */
	isAuthenticating: boolean;
	/** Device code response if auth in progress */
	deviceCode?: DeviceCodeResponse;
	/** Error message if auth failed */
	error?: string;
}

/** Minecraft profile with skins and capes */
export interface MinecraftProfile {
	id: string;
	name: string;
	skins: SkinInfo[];
	capes: CapeInfo[];
}

export interface SkinInfo {
	id: string;
	state: string;
	url: string;
	variant: 'classic' | 'slim';
}

export interface CapeInfo {
	id: string;
	state: string;
	url: string;
	alias: string;
}

/** Request to upload a new skin */
export interface UploadSkinRequest {
	/** "classic" or "slim" */
	variant: 'classic' | 'slim';
	/** Either a URL or base64 encoded image data */
	skinData: string;
	/** Whether skinData is a URL (true) or file data (false) */
	isUrl: boolean;
}
