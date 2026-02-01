#!/usr/bin/env bun
// Cross-platform tauri build wrapper that handles signing errors gracefully

import { spawn, spawnSync } from 'child_process';
import { existsSync, readdirSync } from 'fs';
import { join } from 'path';

const isWindows = process.platform === 'win32';
const isLinux = process.platform === 'linux';

// Set environment variables
process.env.TAURI_BUNDLER_NEW_APPIMAGE_FORMAT = 'true';
if (isLinux) {
	process.env.NO_STRIP = '1';
}

// Force colored output even though we're piping
process.env.FORCE_COLOR = '1';
process.env.CARGO_TERM_COLOR = 'always';

// Run tauri build
const args = ['build', ...process.argv.slice(2)];
const tauriCmd = isWindows ? 'tauri.cmd' : 'tauri';

const child = spawn(tauriCmd, args, {
	stdio: ['inherit', 'pipe', 'pipe'],
	shell: true,
});

let output = '';

const signingErrorFilter = /.*TAURI_SIGNING_PRIVATE_KEY.*|.*public key.*no private key.*/i;

child.stdout?.on('data', (data: Buffer) => {
	const str = data.toString();
	output += str;
	// Filter out signing error messages
	const lines = str.split('\n').filter((line) => !signingErrorFilter.test(line));
	if (lines.length > 0) {
		process.stdout.write(lines.join('\n'));
	}
});

child.stderr?.on('data', (data: Buffer) => {
	const str = data.toString();
	output += str;
	// Filter out signing error messages
	const lines = str.split('\n').filter((line) => !signingErrorFilter.test(line));
	if (lines.length > 0) {
		process.stderr.write(lines.join('\n'));
	}
});

const isSigningError = (output: string): boolean => {
	const signingErrorPatterns = [
		/TAURI_SIGNING_PRIVATE_KEY/i,
		/public key.*no private key/i,
		/private key.*not.*set/i,
		/signing.*private.*key/i,
	];
	return signingErrorPatterns.some((pattern) => pattern.test(output));
};

const hasBundles = (): boolean => {
	const bundleDir = 'src-tauri/target/release/bundle';
	const bundleTypes = ['appimage', 'deb', 'rpm', 'nsis', 'msi', 'dmg', 'macos'];

	for (const type of bundleTypes) {
		const dir = join(bundleDir, type);
		if (existsSync(dir)) {
			try {
				const files = readdirSync(dir);
				if (files.length > 0) {
					return true;
				}
			} catch {
				// Ignore errors reading directory
			}
		}
	}
	return false;
};

const hasAppImage = (): boolean => {
	const appimageDir = 'src-tauri/target/release/bundle/appimage';
	if (!existsSync(appimageDir)) return false;
	try {
		const files = readdirSync(appimageDir);
		return files.some((f) => f.endsWith('.AppImage') && !f.includes('_stripped'));
	} catch {
		return false;
	}
};

const stripAppImage = (): void => {
	const result = spawnSync('bash', ['scripts/strip-appimage.sh'], {
		stdio: 'inherit',
		shell: true,
	});
	if (result.status !== 0) {
		console.error('Warning: Failed to strip AppImage');
	}
};

const onBuildComplete = (success: boolean): void => {
	if (success && isLinux && hasAppImage()) {
		stripAppImage();
	}
	process.exit(success ? 0 : 1);
};

child.on('close', (code: number | null) => {
	if (code === 0) {
		onBuildComplete(true);
		return;
	}

	// Check if the only error was about signing and bundles were created
	if (isSigningError(output) && hasBundles()) {
		console.log('\nBuild succeeded (signing skipped - no TAURI_SIGNING_PRIVATE_KEY set)');
		onBuildComplete(true);
		return;
	}

	// Real error
	onBuildComplete(false);
});
