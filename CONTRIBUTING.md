# Contributing to ETLauncher

Thank you for your interest in contributing to ETLauncher! This document will help you get started with development.

## Prerequisites

### All Platforms

- **Rust 1.92.0+** - Install via [rustup](https://rustup.rs/)
- **Bun** - Install from [bun.sh](https://bun.sh/)
- **Node.js 22** Install from [nodejs.org](https://nodejs.org)

### Platform-Specific Dependencies

#### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

#### Linux (Fedora)

```bash
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
```

#### Linux (Arch)

```bash
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg2
```

#### macOS

```bash
xcode-select --install
```

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++"
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)

## Getting Started

1. **Clone the repository**

   ```bash
   git clone https://github.com/firemonster612/ETlauncher.git
   cd ETlauncher
   ```

2. **Install dependencies**

   ```bash
   bun install
   ```

3. **Run in development mode**

   ```bash
   bun run tauri dev
   ```

   This starts both the Vite dev server and the Tauri application with hot-reload.

4. **Build for production**

   ```bash
   bun run tauri:build
   ```

   Built artifacts will be in `src-tauri/target/release/bundle/`.

   The build script automatically:
   - Uses the portable AppImage format for better Linux compatibility (including Wayland)
   - Sets `NO_STRIP=1` on Linux to avoid issues with outdated strip binaries on non-LTS distros
   - Handles missing signing keys gracefully (signing is only required for releases)
   - Strips the AppImage binary after building on Linux

## Project Structure

```
ETlauncher/
├── src/                    # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   └── services/       # Frontend services
│   └── routes/             # SvelteKit routes
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # Main library entry
│   │   ├── main.rs         # Application entry
│   │   └── services/       # Rust services
│   └── Cargo.toml
├── scripts/                # Build scripts
│   ├── tauri-build.ts      # Cross-platform build wrapper
│   └── strip-appimage.sh   # Linux AppImage stripping
├── static/                 # Static assets
└── package.json
```

## Code Style

### Frontend (TypeScript/Svelte)

```bash
bun run format:check   # Check formatting (Prettier)
bun run format         # Fix formatting
bun run lint           # Check for lint errors (ESLint)
bun run lint:fix       # Fix lint errors
bun run check          # Type check (svelte-check)
```
or more simply: `bun run validate`

### Backend (Rust)

Run these from the `src-tauri/` directory:

```bash
cargo fmt-check        # Check formatting (rustfmt)
cargo fmt              # Fix formatting
cargo lint             # Check for lint errors (Clippy)
cargo check            # Check compilation
```
or more simply: `bun run validate:rs`

## Pull Request Process

1. **Create a branch** from `master` for your changes
2. **Make your changes** and ensure all checks pass:

   ```bash
   # Frontend
   bun run format:check && bun run lint && bun run check

   # Backend (from src-tauri/)
   cargo fmt-check && cargo lint
   ```

3. **Test locally** by running `bun run tauri dev`
4. **Push your branch** and open a pull request
5. **Wait for CI** - All checks must pass before merging

## Commit Messages

Write clear, concise commit messages that explain _what_ and _why_:

- `fix: resolve crash when launching without Java installed`
- `feat: add support for Modrinth modpacks`
- `refactor: simplify instance creation flow`
- `docs: update installation instructions`

## Need Help?

If you have questions or run into issues:

1. Check existing [issues](https://github.com/firemonster612/ETlauncher/issues)
2. Open a new issue if your problem isn't already reported
