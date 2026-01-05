# ETLauncher - Minecraft Launcher Specification

## Overview

ETLauncher is a modern, cross-platform Minecraft launcher built with Tauri and Svelte. It provides a unified interface for managing Minecraft instances, mods, modpacks, and game resources from multiple sources.

## Technology Stack

- **Frontend**: Svelte 5 + SvelteKit (SPA mode)
- **UI Components**: Shadcn-Svelte with Tailwind CSS v4
- **Backend**: Tauri (Rust)
- **Package Manager**: Bun

---

## Core Features

### 1. Instance Management

- Create, edit, delete, and duplicate Minecraft instances
- Each instance is isolated with its own:
  - Minecraft version
  - Mod loader and version
  - Mods, shaders, and resource packs
  - Game settings
  - Java runtime configuration
- Import/export instances
- Instance profiles with custom icons and names

### 2. Modpack Browser (Separate Page)

A dedicated page for browsing and installing modpacks. Selecting a modpack creates a **new instance**.

Supported platforms:

| Platform             | Support Level |
| -------------------- | ------------- |
| Modrinth             | Full          |
| CurseForge           | Full          |
| Feed The Beast (FTB) | Full          |
| Technic              | Full          |
| ATLauncher           | Full          |

Features:

- Browse and search modpacks from each platform
- Filter by Minecraft version, loader, category
- View modpack details, changelogs, screenshots, and versions
- Select modpack version to install
- Creates a new instance with the modpack pre-configured
- Automatic dependency resolution
- Update existing modpack instances to newer versions

### 3. Content Browser (Instance-Level)

Accessed from within an instance via an **"Add Mods"** button (or similar). Used to add content to an **existing instance**.

Browse and install from Modrinth and CurseForge:

- **Mods** - gameplay modifications
- **Shaders** - graphical enhancements
- **Resource Packs** - texture and sound replacements

Features:

- Opens as a modal/panel within the instance view
- Search with filters (loader, version, category)
- Automatically filters to instance's Minecraft version and mod loader
- View content details, descriptions, and screenshots
- Dependency management (auto-install required mods)
- Version compatibility checking
- One-click install directly to the current instance

### 4. Mod Loader Support

Install and manage any mod loader:

| Loader     | Support |
| ---------- | ------- |
| Forge      | Full    |
| NeoForge   | Full    |
| Fabric     | Full    |
| Quilt      | Full    |
| LiteLoader | Full    |

- Automatic mod loader installation
- Version selection per instance
- Loader version updates

### 5. Java Runtime Management

Automatic Java handling:

- Detect existing Java installations
- Download and install appropriate Java versions automatically
- Per-instance Java version selection
- Support for:
  - Java 8 (legacy versions)
  - Java 17 (1.18+)
  - Java 21 (1.20.5+)
- JVM argument customization
- Memory allocation settings (min/max heap)

### 6. Game Console

Built-in console for viewing game output:

- Real-time log streaming during gameplay
- Log level filtering (info, warning, error)
- Search within logs
- Copy log output
- Crash report detection and formatting
- Log export functionality

### 7. Authentication

Support for multiple authentication methods:

- Microsoft Account (primary)
- Offline mode (for testing)
- Account switching
- Multiple account storage

---

## User Interface

### Main Views

2. **Instances** - Grid/list view of all instances
3. **Modpacks** - Browse modpacks from all platforms, create new instances
4. **Settings** - Global launcher settings
5. **Console** - Game output viewer

### Instance View

When selecting an instance, shows:

- Instance details and configuration
- **"Add Mods" button** - Opens the Content Browser to add mods/shaders/resource packs
- Installed mods list with enable/disable toggles
- Installed resource packs with ordering
- Installed shaders with selection
- Screenshots gallery
- World saves
- Instance settings (Java, memory, etc.)

---

## Technical Requirements

### Performance

- Fast startup time
- Efficient resource usage
- Background download management
- Caching for API responses and assets

### Cross-Platform

- Windows
- macOS
- Linux

### Data Storage

- Instance data in user-accessible folder
- Configuration in standard app data location
- Cache management with cleanup options

### Network

- API integration with:
  - Modrinth API
  - CurseForge API (requires API key)
  - FTB API
  - Technic API
  - ATLauncher API
  - Microsoft Authentication
  - Mojang/Minecraft APIs
- Download queue management
- Resume interrupted downloads
- Bandwidth limiting (optional)

---

## Future Considerations

- Mod update checking and batch updates
- Server list integration
- Themes and customization
