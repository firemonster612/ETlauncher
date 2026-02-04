/** A saved skin in the local library */
export interface SavedSkin {
	/** Unique identifier for this skin */
	id: string;
	/** User-given name for the skin */
	name: string;
	/** Skin variant: "classic" (4px arms) or "slim" (3px arms) */
	variant: 'classic' | 'slim';
	/** Filename of the skin in the library */
	filename: string;
	/** Unix timestamp when the skin was saved */
	createdAt: number;
}
