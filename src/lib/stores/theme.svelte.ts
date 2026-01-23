import type {
	Theme,
	ColorPreset,
	ThemeColors,
	FontFamily,
	CustomFont,
	SidebarStyle,
	CustomSidebarColor,
} from '$lib/types';
import { settingsStore } from './settings.svelte';
import * as settingsService from '$lib/services/settings';

/** Color preset definitions */
const COLOR_PRESETS: Record<ColorPreset, { hue: number; chroma: number }> = {
	default: { hue: 172, chroma: 0.18 },
	purple: { hue: 280, chroma: 0.2 },
	green: { hue: 145, chroma: 0.18 },
	orange: { hue: 45, chroma: 0.2 },
	pink: { hue: 330, chroma: 0.22 },
	blue: { hue: 220, chroma: 0.18 },
	custom: { hue: 172, chroma: 0.18 },
};

/** Create the theme store */
function createThemeStore() {
	let currentThemeSetting = $state<Theme>('dark');
	let cachedSystemTheme = $state<'dark' | 'light'>('dark');
	let initialized = false;

	/** Fetch system theme from Tauri backend */
	async function fetchSystemTheme(): Promise<'dark' | 'light'> {
		try {
			const theme = await settingsService.getSystemTheme();
			cachedSystemTheme = theme;
			return theme;
		} catch {
			return 'dark';
		}
	}

	/** Get the effective theme mode (resolves 'system' to actual theme) */
	function getEffectiveTheme(theme: Theme): 'dark' | 'light' {
		if (theme === 'system') {
			return cachedSystemTheme;
		}
		return theme;
	}

	/** Apply the theme mode to the document */
	async function applyThemeMode(theme: Theme) {
		if (typeof document === 'undefined') return;

		currentThemeSetting = theme;

		// Fetch system theme if using 'system' setting
		if (theme === 'system') {
			await fetchSystemTheme();
		}

		const effectiveTheme = getEffectiveTheme(theme);
		const html = document.documentElement;

		if (effectiveTheme === 'dark') {
			html.classList.add('dark');
			html.classList.remove('light');
		} else {
			html.classList.remove('dark');
			html.classList.add('light');
		}
	}

	/** Apply accent color to the document */
	function applyAccentColor(preset: ColorPreset, customColors?: ThemeColors) {
		if (typeof document === 'undefined') return;

		const html = document.documentElement;

		let hue: number;
		let chroma: number;

		if (preset === 'custom' && customColors) {
			hue = customColors.primaryHue ?? 172;
			chroma = customColors.primaryChroma ?? 0.18;
		} else {
			const presetValues = COLOR_PRESETS[preset] ?? COLOR_PRESETS.default;
			hue = presetValues.hue;
			chroma = presetValues.chroma;
		}

		// Determine if we're in dark or light mode
		const isDark = html.classList.contains('dark');

		// Compute the primary color based on current theme
		const primaryLightness = isDark ? 0.72 : 0.55;
		const primary = `oklch(${primaryLightness} ${chroma} ${hue})`;

		// Set the base variables
		html.style.setProperty('--accent-hue', String(hue), 'important');
		html.style.setProperty('--accent-chroma', String(chroma), 'important');

		// Set all primary-related colors with !important to override CSS
		html.style.setProperty('--primary', primary, 'important');
		html.style.setProperty('--ring', primary, 'important');
		html.style.setProperty('--sidebar-primary', primary, 'important');
		html.style.setProperty('--sidebar-accent', primary, 'important');
		html.style.setProperty('--sidebar-ring', primary, 'important');
		html.style.setProperty('--chart-1', primary, 'important');
	}

	/** Handle system preference change */
	async function handleSystemPreferenceChange() {
		// Re-apply theme if using system preference
		if (currentThemeSetting === 'system') {
			await applyThemeMode('system');
			// Re-apply accent color since primary color values differ between light/dark
			const settings = settingsStore.settings;
			if (settings) {
				applyAccentColor(settings.colorPreset ?? 'default', settings.customColors);
				applySidebarStyle(settings.sidebarStyle ?? 'default', settings.customSidebarColor);
			}
		}
	}

	/** Initialize the theme system */
	async function init() {
		if (initialized || typeof window === 'undefined') return;
		initialized = true;

		// Fetch initial system theme from Tauri
		await fetchSystemTheme();

		// Listen for system preference changes via matchMedia as a backup
		// (Tauri may not always fire theme change events)
		if (window.matchMedia) {
			const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
			mediaQuery.addEventListener('change', () => {
				handleSystemPreferenceChange();
			});
		}

		// Poll for theme changes periodically since Tauri's theme change events
		// may not work reliably on all platforms
		setInterval(async () => {
			if (currentThemeSetting === 'system') {
				const newTheme = await settingsService.getSystemTheme();
				if (newTheme !== cachedSystemTheme) {
					cachedSystemTheme = newTheme;
					await handleSystemPreferenceChange();
				}
			}
		}, 1000); // Check every second
	}

	/** Apply hover lift setting */
	function applyHoverLift(disabled: boolean) {
		if (typeof document === 'undefined') return;

		const html = document.documentElement;
		if (disabled) {
			html.classList.add('no-hover-lift');
		} else {
			html.classList.remove('no-hover-lift');
		}
	}

	/** Apply font family setting */
	function applyFontFamily(font: FontFamily, customFont?: CustomFont) {
		if (typeof document === 'undefined') return;

		const html = document.documentElement;
		// Remove all font classes
		html.classList.remove('font-pixel', 'font-system', 'font-custom');

		// Remove any custom font inline style
		html.style.removeProperty('--custom-font-family');

		if (font === 'custom' && customFont?.family) {
			html.classList.add('font-custom');
			html.style.setProperty('--custom-font-family', customFont.family, 'important');
			// Store in localStorage for flash prevention
			localStorage.setItem('etlauncher-custom-font', customFont.family);
		} else if (font !== 'pixel') {
			html.classList.add(`font-${font}`);
			localStorage.removeItem('etlauncher-custom-font');
		} else {
			localStorage.removeItem('etlauncher-custom-font');
		}

		// Store font setting for flash prevention
		localStorage.setItem('etlauncher-font', font);
	}

	/** Apply sidebar style setting */
	function applySidebarStyle(style: SidebarStyle, customColor?: CustomSidebarColor) {
		if (typeof document === 'undefined') return;

		const html = document.documentElement;
		const isDark = html.classList.contains('dark');

		// Get current accent color values
		const accentHue = parseFloat(html.style.getPropertyValue('--accent-hue')) || 172;
		const accentChroma = parseFloat(html.style.getPropertyValue('--accent-chroma')) || 0.18;

		let sidebarColor: string;
		let sidebarForeground: string;
		let sidebarBorder: string;

		switch (style) {
			case 'accent':
				// Use accent color for sidebar
				if (isDark) {
					sidebarColor = `oklch(0.15 ${accentChroma * 0.3} ${accentHue})`;
					sidebarForeground = 'oklch(0.95 0 0)';
					sidebarBorder = `oklch(0.25 ${accentChroma * 0.4} ${accentHue})`;
				} else {
					sidebarColor = `oklch(0.92 ${accentChroma * 0.3} ${accentHue})`;
					sidebarForeground = 'oklch(0.15 0.01 285)';
					sidebarBorder = `oklch(0.85 ${accentChroma * 0.4} ${accentHue})`;
				}
				break;
			case 'custom':
				if (customColor) {
					const { hue, chroma } = customColor;
					if (isDark) {
						sidebarColor = `oklch(0.12 ${chroma} ${hue})`;
						sidebarForeground = 'oklch(0.95 0 0)';
						sidebarBorder = `oklch(0.25 ${chroma} ${hue})`;
					} else {
						sidebarColor = `oklch(0.95 ${chroma} ${hue})`;
						sidebarForeground = 'oklch(0.15 0.01 285)';
						sidebarBorder = `oklch(0.85 ${chroma} ${hue})`;
					}
				} else {
					// Fallback to default
					if (isDark) {
						sidebarColor = 'oklch(0.08 0.015 280)';
						sidebarForeground = 'oklch(0.95 0 0)';
						sidebarBorder = 'oklch(0.25 0.02 280)';
					} else {
						sidebarColor = 'oklch(0.95 0.01 285)';
						sidebarForeground = 'oklch(0.15 0.01 285)';
						sidebarBorder = 'oklch(0.85 0.02 285)';
					}
				}
				break;
			default:
				// 'default' - use CSS default values, remove overrides
				html.style.removeProperty('--sidebar');
				html.style.removeProperty('--sidebar-foreground');
				html.style.removeProperty('--sidebar-border');
				localStorage.setItem('etlauncher-sidebar-style', 'default');
				localStorage.removeItem('etlauncher-sidebar-color');
				return;
		}

		html.style.setProperty('--sidebar', sidebarColor, 'important');
		html.style.setProperty('--sidebar-foreground', sidebarForeground, 'important');
		html.style.setProperty('--sidebar-border', sidebarBorder, 'important');

		// Store for flash prevention
		localStorage.setItem('etlauncher-sidebar-style', style);
		if (style === 'custom' && customColor) {
			localStorage.setItem('etlauncher-sidebar-color', JSON.stringify(customColor));
		} else {
			localStorage.removeItem('etlauncher-sidebar-color');
		}
	}

	/** Apply all theme settings from the settings store */
	async function applyFromSettings() {
		const settings = settingsStore.settings;
		if (!settings) return;

		await applyThemeMode(settings.theme);
		applyAccentColor(settings.colorPreset ?? 'default', settings.customColors);
		applyHoverLift(settings.disableHoverLift ?? false);
		applyFontFamily(settings.fontFamily ?? 'pixel', settings.customFont);
		applySidebarStyle(settings.sidebarStyle ?? 'default', settings.customSidebarColor);
	}

	return {
		get systemPrefersDark() {
			return cachedSystemTheme === 'dark';
		},

		/** Initialize the theme system (call once on app mount) */
		init,

		/** Apply theme mode */
		applyThemeMode,

		/** Apply accent color */
		applyAccentColor,

		/** Apply hover lift setting */
		applyHoverLift,

		/** Apply font family setting */
		applyFontFamily,

		/** Apply sidebar style setting */
		applySidebarStyle,

		/** Apply all theme settings from settings store */
		applyFromSettings,

		/** Get the effective theme (resolves 'system') */
		getEffectiveTheme,

		/** Get color preset values */
		getPresetValues(preset: ColorPreset): { hue: number; chroma: number } {
			return COLOR_PRESETS[preset] ?? COLOR_PRESETS.default;
		},

		/** Get all available presets */
		get presets(): ColorPreset[] {
			return ['default', 'purple', 'green', 'orange', 'pink', 'blue', 'custom'];
		},
	};
}

/** Global theme store instance */
export const themeStore = createThemeStore();
