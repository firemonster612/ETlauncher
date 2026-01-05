# ETLauncher Development Roadmap

## Overview

A Minecraft launcher built with Tauri + SvelteKit that supports multiple modpack platforms, mod loaders, and automatic Java management.

---

## Phase 1 - Foundation (Current)

**Goal:** UI shell and basic structure

- [ ] Custom titlebar (no native decorations)
- [ ] Sidebar navigation
- [ ] Route structure for all views
- [ ] Dark theme with design language
- [ ] Placeholder pages for all views

**Views to create:**

- Home/Dashboard
- Instances
- Modpacks
- Downloads
- Console
- Settings

---

## Phase 2 - Core Functionality

**Goal:** Basic Minecraft launching capability

- [ ] Microsoft authentication (OAuth flow)
- [ ] Account storage and switching
- [ ] Instance management (create, edit, delete, duplicate)
- [ ] Instance storage structure
- [ ] Minecraft version manifest fetching
- [ ] Game asset downloading (client jar, assets, libraries)
- [ ] Launch game process
- [ ] Basic settings storage (paths, memory allocation)

---

## Phase 3 - Mod Loaders

**Goal:** Support for all major mod loaders

- [ ] Forge installer integration
- [ ] NeoForge installer integration
- [ ] Fabric installer integration
- [ ] Quilt installer integration
- [ ] LiteLoader support (legacy)
- [ ] Mod loader version fetching
- [ ] Per-instance mod loader configuration

---

## Phase 4 - Content Integration

**Goal:** Browse and install content from platforms

### Modpack Browser (Separate Page)

- [ ] Modrinth modpack API integration
- [ ] CurseForge modpack API integration
- [ ] Feed The Beast (FTB) API integration
- [ ] Technic API integration
- [ ] ATLauncher API integration
- [ ] Modpack search and filtering
- [ ] Create instance from modpack
- [ ] Modpack version selection
- [ ] Update existing modpack instances

### Content Browser (Instance-Level)

- [ ] Modrinth mod/shader/resourcepack API
- [ ] CurseForge mod/shader/resourcepack API
- [ ] Content search with filters
- [ ] Auto-filter by instance version/loader
- [ ] Dependency resolution
- [ ] One-click install to instance

---

## Phase 5 - Java Management

**Goal:** Automatic Java runtime handling

- [ ] Detect existing Java installations
- [ ] Download appropriate Java versions (Adoptium/Temurin)
- [ ] Java 8 support (legacy Minecraft)
- [ ] Java 17 support (1.18+)
- [ ] Java 21 support (1.20.5+)
- [ ] Per-instance Java selection
- [ ] JVM argument customization
- [ ] Memory allocation settings

---

## Phase 6 - Instance Details

**Goal:** Full instance management UI

- [ ] Instance overview page
- [ ] Installed mods list (enable/disable)
- [ ] Resource packs management (ordering)
- [ ] Shaders management (selection)
- [ ] Screenshots gallery
- [ ] Instance settings (Java, memory, JVM args)
- [ ] Instance export/import

---

## Phase 9 - Polish & Features

**Goal:** Quality of life improvements

- [ ] Mod update checking
- [ ] Batch mod updates
- [ ] Instance search/filter
- [ ] Keyboard shortcuts
- [ ] Notifications (toast/sonner)
- [ ] First-run setup wizard
- [ ] Error handling improvements
- [ ] Loading states and skeletons
- [ ] for mods that block API opening the download page for the mod automatically then getting the mod from the user's downloads folder
- [ ] metadata retrival from CurseForge or Modrinth if mods are unknown

---

## Future Considerations

- Server list integration
- Themes and customization
- Offline mode improvements
- Performance profiling tools

---

## Technical Stack

| Layer           | Technology                          |
| --------------- | ----------------------------------- |
| Frontend        | Svelte 5, SvelteKit, Tailwind CSS 4 |
| UI Components   | shadcn-svelte                       |
| Backend         | Tauri (Rust)                        |
| Package Manager | Bun                                 |
| Icons           | Lucide                              |

---

## API Integrations Required

| Service          | Purpose                 | Auth Required    |
| ---------------- | ----------------------- | ---------------- |
| Microsoft        | Player authentication   | OAuth            |
| Mojang/Minecraft | Game assets, versions   | Via MS auth      |
| Modrinth         | Mods, modpacks, shaders | API key optional |
| CurseForge       | Mods, modpacks, shaders | API key required |
| FTB              | Modpacks                | None             |
| Technic          | Modpacks                | None             |
| ATLauncher       | Modpacks                | None             |
| Adoptium         | Java downloads          | None             |
