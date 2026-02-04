<script lang="ts">
  import { onboardingStore } from "$lib/stores/onboarding.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Button } from "$lib/ui/button";
  import ColorPicker from "$lib/components/ColorPicker.svelte";
  import {
    ChevronLeft,
    ChevronRight,
    X,
    Gamepad2,
    Check,
    Copy,
    ExternalLink,
    Loader2,
    Monitor,
    Sun,
    Moon,
    Palette,
    Boxes,
    Package,
    Puzzle,
    Users,
    Terminal,
    Settings,
    Key,
    Eye,
    EyeOff,
    RefreshCw,
    Shirt,
  } from "@lucide/svelte";
  import type {
    Theme,
    ColorPreset,
    ThemeColors,
    FontFamily,
    SidebarStyle,
  } from "$lib/types";

  // Local state for appearance settings
  let selectedTheme = $state<Theme>("dark");
  let selectedColorPreset = $state<ColorPreset>("default");
  let customHue = $state(172);
  let customChroma = $state(0.18);
  let selectedFont = $state<FontFamily>("pixel");
  let customFontFamily = $state("Arial");
  let selectedSidebarStyle = $state<SidebarStyle>("default");
  let sidebarHue = $state(220);
  let sidebarChroma = $state(0.05);

  // Login state
  let copiedCode = $state(false);

  // CurseForge API key state
  let curseforgeApiKey = $state("");
  let showApiKey = $state(false);

  // Auto-update state
  let autoUpdate = $state(true);
  let includePreReleases = $state(false);

  // Features carousel state
  let currentFeatureIndex = $state(0);

  // Color presets
  const colorPresets: {
    id: ColorPreset;
    label: string;
    hue: number;
    chroma: number;
  }[] = [
    { id: "default", label: "Cyan", hue: 172, chroma: 0.18 },
    { id: "purple", label: "Purple", hue: 280, chroma: 0.2 },
    { id: "green", label: "Green", hue: 145, chroma: 0.18 },
    { id: "orange", label: "Orange", hue: 45, chroma: 0.2 },
    { id: "pink", label: "Pink", hue: 330, chroma: 0.22 },
    { id: "blue", label: "Blue", hue: 220, chroma: 0.18 },
  ];

  // Features list
  const features = [
    {
      icon: Boxes,
      title: "Instances",
      description:
        "Create and manage Minecraft installations with different versions and mod loaders",
    },
    {
      icon: Package,
      title: "Modpacks",
      description:
        "Browse and install modpacks from Modrinth, CurseForge, FTB, and Technic",
    },
    {
      icon: Puzzle,
      title: "Content Browser",
      description: "Add mods, shaders, and resource packs to any instance",
    },
    {
      icon: Users,
      title: "Accounts",
      description: "Manage multiple Microsoft accounts",
    },
    {
      icon: Shirt,
      title: "Skin Changer",
      description:
        "Change your Minecraft skin and cape, with a skin library for managing your skins.",
    },
    {
      icon: Terminal,
      title: "Console",
      description: "Monitor running games and view logs",
    },
    {
      icon: Settings,
      title: "Settings",
      description: "Configure memory, downloads, and more",
    },
  ];

  // Initialize from settings when wizard opens
  $effect(() => {
    if (onboardingStore.isOpen && settingsStore.settings) {
      const s = settingsStore.settings;
      selectedTheme = s.theme;
      selectedColorPreset = s.colorPreset ?? "default";
      if (s.customColors) {
        customHue = s.customColors.primaryHue ?? 172;
        customChroma = s.customColors.primaryChroma ?? 0.18;
      }
      selectedFont = s.fontFamily ?? "pixel";
      if (s.customFont?.family) {
        customFontFamily = s.customFont.family;
      }
      selectedSidebarStyle = s.sidebarStyle ?? "default";
      if (s.customSidebarColor) {
        sidebarHue = s.customSidebarColor.hue ?? 220;
        sidebarChroma = s.customSidebarColor.chroma ?? 0.05;
      }
      curseforgeApiKey = s.curseforgeApiKey ?? "";
      autoUpdate = s.autoUpdate ?? true;
      includePreReleases = s.includePreReleases ?? false;
    }
  });

  // Watch for login completion
  $effect(() => {
    if (
      onboardingStore.currentStep === "login" &&
      accountsStore.accounts.length > 0 &&
      !accountsStore.isAuthenticating
    ) {
      onboardingStore.setLoginCompleted(true);
    }
  });

  // Theme handlers
  async function handleThemeChange(theme: Theme) {
    selectedTheme = theme;
    await themeStore.applyThemeMode(theme);
    themeStore.applyAccentColor(selectedColorPreset, getCustomColors());
    themeStore.applySidebarStyle(selectedSidebarStyle, getCustomSidebarColor());
  }

  function handleColorPresetChange(preset: ColorPreset) {
    selectedColorPreset = preset;
    themeStore.applyAccentColor(
      preset,
      preset === "custom" ? getCustomColors() : undefined,
    );
  }

  function handleCustomColorChange(hue: number, chroma: number) {
    customHue = hue;
    customChroma = Math.max(0.05, chroma * 0.35);
    themeStore.applyAccentColor("custom", {
      primaryHue: customHue,
      primaryChroma: customChroma,
    });
  }

  function handleFontChange(font: FontFamily) {
    selectedFont = font;
    if (font === "custom") {
      themeStore.applyFontFamily("custom", { family: customFontFamily });
    } else {
      themeStore.applyFontFamily(font);
    }
  }

  function handleCustomFontChange(family: string) {
    customFontFamily = family;
    themeStore.applyFontFamily("custom", { family });
  }

  function handleSidebarStyleChange(style: SidebarStyle) {
    selectedSidebarStyle = style;
    themeStore.applySidebarStyle(
      style,
      style === "custom" ? getCustomSidebarColor() : undefined,
    );
  }

  function handleCustomSidebarColorChange(hue: number, chroma: number) {
    sidebarHue = hue;
    sidebarChroma = chroma * 0.35;
    themeStore.applySidebarStyle("custom", {
      hue: sidebarHue,
      chroma: sidebarChroma,
    });
  }

  function getCustomColors(): ThemeColors {
    return { primaryHue: customHue, primaryChroma: customChroma };
  }

  function getCustomSidebarColor() {
    return { hue: sidebarHue, chroma: sidebarChroma };
  }

  // Login handlers
  async function startLogin() {
    await accountsStore.startAuth();
  }

  async function copyCode() {
    if (accountsStore.deviceCode) {
      await navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
      copiedCode = true;
      setTimeout(() => (copiedCode = false), 2000);
    }
  }

  async function copyAndOpen() {
    if (accountsStore.deviceCode) {
      await navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
      copiedCode = true;
      setTimeout(() => (copiedCode = false), 2000);
      await openUrl(accountsStore.deviceCode.verificationUri);
    }
  }

  function skipLogin() {
    accountsStore.stopAuth();
    onboardingStore.next();
  }

  // Navigation handlers
  async function handleNext() {
    if (onboardingStore.isLastStep) {
      await completeOnboarding();
    } else {
      onboardingStore.next();
    }
  }

  function handleBack() {
    onboardingStore.back();
  }

  async function handleSkip() {
    accountsStore.stopAuth();
    await completeOnboarding();
  }

  async function completeOnboarding() {
    // Save all settings
    await settingsStore.update({
      theme: selectedTheme,
      colorPreset: selectedColorPreset,
      customColors:
        selectedColorPreset === "custom" ? getCustomColors() : undefined,
      fontFamily: selectedFont,
      customFont:
        selectedFont === "custom" ? { family: customFontFamily } : undefined,
      sidebarStyle: selectedSidebarStyle,
      customSidebarColor:
        selectedSidebarStyle === "custom" ? getCustomSidebarColor() : undefined,
      curseforgeApiKey: curseforgeApiKey.trim() || undefined,
      autoUpdate,
      includePreReleases,
      setupCompleted: true,
    });

    // Save to localStorage for flash prevention
    try {
      localStorage.setItem("etlauncher-theme", JSON.stringify(selectedTheme));
      const presetValues = themeStore.getPresetValues(selectedColorPreset);
      const hue =
        selectedColorPreset === "custom" ? customHue : presetValues.hue;
      const chroma =
        selectedColorPreset === "custom" ? customChroma : presetValues.chroma;
      localStorage.setItem(
        "etlauncher-accent",
        JSON.stringify({ hue, chroma }),
      );
      localStorage.setItem("etlauncher-font", selectedFont);
      if (selectedFont === "custom") {
        localStorage.setItem("etlauncher-custom-font", customFontFamily);
      }
      localStorage.setItem("etlauncher-sidebar-style", selectedSidebarStyle);
      if (selectedSidebarStyle === "custom") {
        localStorage.setItem(
          "etlauncher-sidebar-color",
          JSON.stringify({ hue: sidebarHue, chroma: sidebarChroma }),
        );
      }
    } catch {
      // localStorage may not be available
    }

    onboardingStore.complete();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!onboardingStore.isOpen) return;
    if (e.key === "Escape") handleSkip();
    if (e.key === "Enter" && !onboardingStore.isFirstStep) handleNext();
  }

  // Features carousel navigation
  function nextFeature() {
    currentFeatureIndex = (currentFeatureIndex + 1) % features.length;
  }

  function prevFeature() {
    currentFeatureIndex =
      (currentFeatureIndex - 1 + features.length) % features.length;
  }

  function goToFeature(index: number) {
    currentFeatureIndex = index;
  }

  // Get step number for display (1-indexed, login step is special)
  const stepLabels = [
    "Welcome",
    "Sign In",
    "Theme",
    "Font",
    "CurseForge",
    "Updates",
    "Features",
    "Done",
  ];

  // CurseForge handlers
  async function openCurseForgeConsole() {
    await openUrl("https://console.curseforge.com");
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if onboardingStore.isOpen}
  <div
    class="fixed top-[var(--titlebar-height)] right-0 bottom-0 left-0 z-50 flex items-center justify-center bg-black/60 p-6"
    role="dialog"
    aria-label="Onboarding Wizard"
  >
    <div
      class="bg-card border-border animate-in fade-in zoom-in-95 relative flex h-[min(85vh,700px)] w-[min(90vw,600px)] flex-col overflow-hidden rounded-lg border-2 shadow-2xl duration-200"
    >
      <!-- Header -->
      <div
        class="border-border flex items-center justify-between border-b px-6 py-4"
      >
        <div class="flex items-center gap-3">
          <span class="text-muted-foreground text-sm">
            {stepLabels[onboardingStore.currentStepIndex]}
          </span>
        </div>
        <button
          onclick={handleSkip}
          class="text-muted-foreground hover:text-foreground transition-colors"
          aria-label="Skip onboarding"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Progress dots -->
      <div class="flex items-center justify-center gap-1.5 py-3">
        {#each Array.from({ length: onboardingStore.totalSteps }, (_, i) => i) as i (i)}
          <div
            class="h-2 w-2 rounded-full transition-all duration-300"
            class:bg-primary={i === onboardingStore.currentStepIndex}
            class:scale-125={i === onboardingStore.currentStepIndex}
            class:bg-muted={i !== onboardingStore.currentStepIndex}
          ></div>
        {/each}
      </div>

      <!-- Content area with step transitions -->
      <div class="flex-1 overflow-y-auto px-6 pb-4">
        {#if onboardingStore.currentStep === "welcome"}
          <!-- Welcome Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div
              class="bg-primary/10 mx-auto flex h-20 w-20 items-center justify-center rounded-2xl"
            >
              <Gamepad2 class="text-primary h-10 w-10" />
            </div>

            <div class="text-center">
              <h2 class="text-2xl font-bold">Welcome to ETLauncher</h2>
              <p class="text-muted-foreground mt-2">
                Your all-in-one Minecraft launcher for managing instances,
                modpacks, and more.
              </p>
            </div>

            <div class="bg-muted/50 space-y-2 rounded-lg p-4">
              <p class="text-sm font-medium">In this setup, you'll:</p>
              <ul class="text-muted-foreground space-y-1 text-sm">
                <li class="flex items-center gap-2">
                  <Check class="text-primary h-4 w-4" />
                  Sign in with your Microsoft account
                </li>
                <li class="flex items-center gap-2">
                  <Check class="text-primary h-4 w-4" />
                  Customize the launcher's appearance
                </li>
                <li class="flex items-center gap-2">
                  <Check class="text-primary h-4 w-4" />
                  Learn about the main features
                </li>
              </ul>
            </div>
          </div>
        {:else if onboardingStore.currentStep === "login"}
          <!-- Login Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-4 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">Sign In</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Sign in with your Microsoft account to play Minecraft
              </p>
            </div>

            {#if accountsStore.accounts.length > 0}
              <!-- Already logged in -->
              <div class="py-4 text-center">
                <Check class="mx-auto mb-3 h-12 w-12 text-green-500" />
                <p class="font-medium">Already signed in!</p>
                <p class="text-muted-foreground mt-1 text-sm">
                  Welcome, {accountsStore.activeAccount?.username ||
                    accountsStore.accounts[0]?.username}
                </p>
              </div>
            {:else if !accountsStore.isAuthenticating}
              <!-- Not started yet -->
              <div class="py-4 text-center">
                <p class="text-muted-foreground mb-4 text-sm">
                  You need a Microsoft account with Minecraft to play.
                </p>
                <Button onclick={startLogin} class="w-full"
                  >Sign in with Microsoft</Button
                >
              </div>
            {:else if accountsStore.deviceCode}
              <!-- Device code flow -->
              <div class="space-y-4">
                <div class="text-center">
                  <p class="text-muted-foreground mb-2 text-sm">
                    Enter this code at Microsoft:
                  </p>
                  <div
                    class="bg-muted rounded-lg p-4 font-mono text-2xl font-bold tracking-widest"
                  >
                    {accountsStore.deviceCode.userCode}
                  </div>
                </div>

                <div class="flex gap-2">
                  <Button variant="outline" onclick={copyCode} class="flex-1">
                    {#if copiedCode}
                      <Check class="mr-2 h-4 w-4" />
                      Copied!
                    {:else}
                      <Copy class="mr-2 h-4 w-4" />
                      Copy Code
                    {/if}
                  </Button>
                  <Button onclick={copyAndOpen} class="flex-1">
                    <ExternalLink class="mr-2 h-4 w-4" />
                    Open & Sign In
                  </Button>
                </div>

                <div
                  class="text-muted-foreground flex items-center justify-center gap-2 text-sm"
                >
                  <Loader2 class="h-4 w-4 animate-spin" />
                  <span>Waiting for sign in...</span>
                </div>
              </div>
            {:else}
              <!-- Loading -->
              <div class="flex items-center justify-center py-8">
                <Loader2 class="text-primary h-8 w-8 animate-spin" />
              </div>
            {/if}

            {#if accountsStore.authError}
              <div
                class="bg-destructive/10 border-destructive text-destructive rounded border p-3 text-sm"
              >
                {accountsStore.authError}
              </div>
            {/if}
          </div>
        {:else if onboardingStore.currentStep === "theme"}
          <!-- Theme & Accent Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">Choose Your Theme</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Pick your preferred color scheme and accent color
              </p>
            </div>

            <!-- Theme Mode -->
            <div class="space-y-2">
              <span class="text-sm font-medium">Theme Mode</span>
              <div class="grid grid-cols-3 gap-2">
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedTheme ===
                  'system'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleThemeChange("system")}
                >
                  <Monitor class="h-6 w-6" />
                  <span class="text-sm">System</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedTheme ===
                  'light'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleThemeChange("light")}
                >
                  <Sun class="h-6 w-6" />
                  <span class="text-sm">Light</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedTheme ===
                  'dark'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleThemeChange("dark")}
                >
                  <Moon class="h-6 w-6" />
                  <span class="text-sm">Dark</span>
                </button>
              </div>
            </div>

            <!-- Accent Color -->
            <div class="space-y-2">
              <span class="text-sm font-medium">Accent Color</span>
              <div class="grid grid-cols-4 gap-2 sm:grid-cols-7">
                {#each colorPresets as preset (preset.id)}
                  {@const isSelected = selectedColorPreset === preset.id}
                  <button
                    class="flex flex-col items-center gap-1 rounded-lg border-2 p-2 transition-all {isSelected
                      ? 'border-primary bg-primary/10'
                      : 'border-border hover:border-primary/50'}"
                    onclick={() => handleColorPresetChange(preset.id)}
                    title={preset.label}
                  >
                    <div
                      class="ring-offset-background h-6 w-6 rounded-full ring-2 ring-offset-2 {isSelected
                        ? 'ring-primary'
                        : 'ring-transparent'}"
                      style="background: oklch(0.65 {preset.chroma} {preset.hue})"
                    ></div>
                    <span class="text-xs">{preset.label}</span>
                  </button>
                {/each}
                <button
                  class="flex flex-col items-center gap-1 rounded-lg border-2 p-2 transition-all {selectedColorPreset ===
                  'custom'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleColorPresetChange("custom")}
                  title="Custom"
                >
                  <Palette
                    class="h-6 w-6 {selectedColorPreset === 'custom'
                      ? 'text-primary'
                      : ''}"
                  />
                  <span class="text-xs">Custom</span>
                </button>
              </div>
            </div>

            <!-- Custom Color Picker -->
            {#if selectedColorPreset === "custom"}
              <div class="bg-muted/50 flex items-center gap-3 rounded-lg p-3">
                <span class="text-sm">Custom color:</span>
                <ColorPicker
                  hue={customHue}
                  saturation={customChroma / 0.35}
                  oninput={(h, s) => handleCustomColorChange(h, s)}
                  onchange={(h, s) => handleCustomColorChange(h, s)}
                />
              </div>
            {/if}
          </div>
        {:else if onboardingStore.currentStep === "font"}
          <!-- Font & Sidebar Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">Customize Appearance</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Choose your font and sidebar style
              </p>
            </div>

            <!-- Font Selection -->
            <div class="space-y-2">
              <span class="text-sm font-medium">Font</span>
              <div class="grid grid-cols-3 gap-2">
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedFont ===
                  'pixel'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleFontChange("pixel")}
                >
                  <span
                    class="text-lg"
                    style="font-family: 'Silkscreen', monospace">Aa</span
                  >
                  <span class="text-sm">Pixel</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedFont ===
                  'system'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleFontChange("system")}
                >
                  <span
                    class="text-lg"
                    style="font-family: system-ui, sans-serif">Aa</span
                  >
                  <span class="text-sm">System</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedFont ===
                  'custom'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleFontChange("custom")}
                >
                  <span
                    class="text-lg"
                    style="font-family: {customFontFamily}, sans-serif">Aa</span
                  >
                  <span class="text-sm">Custom</span>
                </button>
              </div>
              {#if selectedFont === "custom"}
                <input
                  type="text"
                  class="border-border bg-background text-foreground focus:border-primary w-full rounded-lg border-2 px-3 py-2 text-sm focus:outline-none"
                  placeholder="Font name (e.g., Arial, Roboto)"
                  value={customFontFamily}
                  oninput={(e) => handleCustomFontChange(e.currentTarget.value)}
                />
              {/if}
            </div>

            <!-- Sidebar Style -->
            <div class="space-y-2">
              <span class="text-sm font-medium">Sidebar Style</span>
              <div class="grid grid-cols-3 gap-2">
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedSidebarStyle ===
                  'default'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleSidebarStyleChange("default")}
                >
                  <div
                    class="bg-muted border-border h-6 w-10 rounded border"
                  ></div>
                  <span class="text-sm">Default</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedSidebarStyle ===
                  'accent'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleSidebarStyleChange("accent")}
                >
                  <div
                    class="bg-primary/20 border-primary/40 h-6 w-10 rounded border"
                  ></div>
                  <span class="text-sm">Accent</span>
                </button>
                <button
                  class="flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-all {selectedSidebarStyle ===
                  'custom'
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'}"
                  onclick={() => handleSidebarStyleChange("custom")}
                >
                  <div
                    class="h-6 w-10 rounded border"
                    style="background: oklch(0.2 {sidebarChroma} {sidebarHue}); border-color: oklch(0.35 {sidebarChroma} {sidebarHue})"
                  ></div>
                  <span class="text-sm">Custom</span>
                </button>
              </div>
              {#if selectedSidebarStyle === "custom"}
                <div class="bg-muted/50 flex items-center gap-3 rounded-lg p-3">
                  <span class="text-sm">Sidebar color:</span>
                  <ColorPicker
                    hue={sidebarHue}
                    saturation={sidebarChroma / 0.35}
                    oninput={(h, s) => handleCustomSidebarColorChange(h, s)}
                    onchange={(h, s) => handleCustomSidebarColorChange(h, s)}
                  />
                </div>
              {/if}
            </div>
          </div>
        {:else if onboardingStore.currentStep === "curseforge"}
          <!-- CurseForge API Key Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">CurseForge API Key</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Optional: Add your API key to browse CurseForge content
              </p>
            </div>

            <div
              class="bg-primary/10 mx-auto flex h-16 w-16 items-center justify-center rounded-full"
            >
              <Key class="text-primary h-8 w-8" />
            </div>

            <div class="space-y-4">
              <div class="space-y-2">
                <label for="curseforge-api-key" class="text-sm font-medium"
                  >API Key</label
                >
                <div class="flex gap-2">
                  <input
                    id="curseforge-api-key"
                    type={showApiKey ? "text" : "password"}
                    class="border-border bg-background text-foreground focus:border-primary flex-1 rounded-lg border-2 px-3 py-2 text-sm focus:outline-none"
                    placeholder="Enter your CurseForge API key"
                    bind:value={curseforgeApiKey}
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onclick={() => (showApiKey = !showApiKey)}
                    aria-label={showApiKey ? "Hide API key" : "Show API key"}
                  >
                    {#if showApiKey}
                      <EyeOff class="h-4 w-4" />
                    {:else}
                      <Eye class="h-4 w-4" />
                    {/if}
                  </Button>
                </div>
              </div>

              <div class="bg-muted/50 space-y-3 rounded-lg p-4">
                <p class="text-sm">
                  A CurseForge API key is required to browse and download
                  content from CurseForge.
                </p>
                <p class="text-muted-foreground text-sm">
                  A built-in API key is planned for a future release. For now,
                  you'll need to get your own free key from CurseForge.
                </p>
                <Button
                  variant="link"
                  onclick={openCurseForgeConsole}
                  class="w-full"
                >
                  <ExternalLink class="mr-2 h-4 w-4" />
                  Get Free API Key
                </Button>
              </div>

              <p class="text-muted-foreground text-center text-xs">
                You can skip this step and add your API key later in Settings.
              </p>
            </div>
          </div>
        {:else if onboardingStore.currentStep === "updates"}
          <!-- Updates Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">Automatic Updates</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Keep ETLauncher up to date with the latest features and fixes
              </p>
            </div>

            <div
              class="bg-primary/10 mx-auto flex h-16 w-16 items-center justify-center rounded-full"
            >
              <RefreshCw class="text-primary h-8 w-8" />
            </div>

            <div class="space-y-4">
              <!-- Auto Update Toggle -->
              <button
                class="flex w-full items-center justify-between rounded-lg border-2 p-4 text-left transition-all {autoUpdate
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'}"
                onclick={() => (autoUpdate = !autoUpdate)}
              >
                <div class="flex-1">
                  <span class="text-sm font-medium">Automatic Updates</span>
                  <p class="text-muted-foreground mt-0.5 text-xs">
                    Check for updates when the launcher starts
                  </p>
                </div>
                <div
                  class="flex h-6 w-6 items-center justify-center rounded-full {autoUpdate
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted'}"
                >
                  {#if autoUpdate}
                    <Check class="h-4 w-4" />
                  {/if}
                </div>
              </button>

              <!-- Pre-release Toggle -->
              <button
                class="flex w-full items-center justify-between rounded-lg border-2 p-4 text-left transition-all {includePreReleases
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'}"
                onclick={() => (includePreReleases = !includePreReleases)}
              >
                <div class="flex-1">
                  <span class="text-sm font-medium">Include Pre-releases</span>
                  <p class="text-muted-foreground mt-0.5 text-xs">
                    Receive alpha, beta, and release candidate versions
                  </p>
                </div>
                <div
                  class="flex h-6 w-6 items-center justify-center rounded-full {includePreReleases
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted'}"
                >
                  {#if includePreReleases}
                    <Check class="h-4 w-4" />
                  {/if}
                </div>
              </button>

              <p class="text-muted-foreground text-center text-xs">
                You can change these settings anytime in the Settings page.
              </p>
            </div>
          </div>
        {:else if onboardingStore.currentStep === "features"}
          <!-- Features Overview Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div class="text-center">
              <h2 class="text-xl font-bold">Feature Overview</h2>
              <p class="text-muted-foreground mt-1 text-sm">
                Here's what you can do with ETLauncher
              </p>
            </div>

            <!-- Feature Carousel -->
            <div class="relative">
              <div class="bg-muted/50 rounded-lg p-6">
                {#each [features[currentFeatureIndex]] as feature (feature.title)}
                  <div class="flex flex-col items-center gap-4 text-center">
                    <div
                      class="bg-primary/10 flex h-16 w-16 items-center justify-center rounded-full"
                    >
                      <feature.icon class="text-primary h-8 w-8" />
                    </div>
                    <div>
                      <h3 class="text-lg font-semibold">{feature.title}</h3>
                      <p class="text-muted-foreground mt-1 text-sm">
                        {feature.description}
                      </p>
                    </div>
                  </div>
                {/each}
              </div>

              <!-- Carousel Controls -->
              <div class="mt-4 flex items-center justify-between">
                <Button variant="ghost" size="icon" onclick={prevFeature}>
                  <ChevronLeft class="h-5 w-5" />
                </Button>
                <div class="flex gap-1.5">
                  {#each features as feature, i (feature.title)}
                    <button
                      class="h-2 w-2 rounded-full transition-all {i ===
                      currentFeatureIndex
                        ? 'bg-primary scale-125'
                        : 'bg-muted hover:bg-muted-foreground/50'}"
                      onclick={() => goToFeature(i)}
                      aria-label="Go to feature {i + 1}"
                    ></button>
                  {/each}
                </div>
                <Button variant="ghost" size="icon" onclick={nextFeature}>
                  <ChevronRight class="h-5 w-5" />
                </Button>
              </div>
            </div>

            <!-- Quick feature list -->
            <div class="grid grid-cols-3 gap-2">
              {#each features as feature, i (i)}
                <button
                  class="flex flex-col items-center gap-1 rounded-lg border p-2 transition-all {i ===
                  currentFeatureIndex
                    ? 'border-primary bg-primary/10'
                    : 'hover:bg-muted/50 border-transparent'}"
                  onclick={() => goToFeature(i)}
                >
                  <feature.icon class="h-4 w-4" />
                  <span class="text-xs">{feature.title}</span>
                </button>
              {/each}
            </div>
          </div>
        {:else if onboardingStore.currentStep === "complete"}
          <!-- Completion Step -->
          <div
            class="animate-in fade-in slide-in-from-right-4 space-y-6 duration-300"
          >
            <div
              class="bg-primary/10 mx-auto flex h-20 w-20 items-center justify-center rounded-full"
            >
              <Check class="text-primary h-10 w-10" />
            </div>

            <div class="text-center">
              <h2 class="text-2xl font-bold">You're All Set!</h2>
              <p class="text-muted-foreground mt-2">
                ETLauncher is ready to use. Here's a summary of your setup:
              </p>
            </div>

            <!-- Setup Summary -->
            <div class="bg-muted/50 space-y-3 rounded-lg p-4">
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Account</span>
                <span class="font-medium">
                  {#if accountsStore.accounts.length > 0}
                    {accountsStore.activeAccount?.username ||
                      accountsStore.accounts[0]?.username}
                  {:else}
                    Not signed in
                  {/if}
                </span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Theme</span>
                <span class="font-medium capitalize">{selectedTheme}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Accent</span>
                <span class="font-medium capitalize">{selectedColorPreset}</span
                >
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Font</span>
                <span class="font-medium capitalize">{selectedFont}</span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Sidebar</span>
                <span class="font-medium capitalize"
                  >{selectedSidebarStyle}</span
                >
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">CurseForge</span>
                <span class="font-medium">
                  {#if curseforgeApiKey.trim()}
                    API key set
                  {:else}
                    Not configured
                  {/if}
                </span>
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Auto-update</span>
                <span class="font-medium"
                  >{autoUpdate ? "Enabled" : "Disabled"}</span
                >
              </div>
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">Pre-releases</span>
                <span class="font-medium"
                  >{includePreReleases ? "Enabled" : "Disabled"}</span
                >
              </div>
            </div>

            <p class="text-muted-foreground text-center text-sm">
              You can change these settings anytime in the Settings page.
            </p>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="border-border flex items-center justify-between border-t px-6 py-4"
      >
        {#if onboardingStore.currentStep === "login"}
          <Button variant="ghost" size="sm" onclick={skipLogin}
            >Skip for now</Button
          >
          {#if accountsStore.accounts.length > 0 || onboardingStore.loginCompleted}
            <Button size="sm" onclick={handleNext}>
              Continue
              <ChevronRight class="ml-1 h-4 w-4" />
            </Button>
          {/if}
        {:else}
          <Button
            variant="ghost"
            size="sm"
            onclick={handleBack}
            disabled={onboardingStore.isFirstStep}
          >
            <ChevronLeft class="mr-1 h-4 w-4" />
            Back
          </Button>
          <Button size="sm" onclick={handleNext}>
            {#if onboardingStore.isLastStep}
              Start Using ETLauncher
            {:else}
              {onboardingStore.currentStep === "welcome"
                ? "Get Started"
                : "Next"}
              {#if !onboardingStore.isLastStep}
                <ChevronRight class="ml-1 h-4 w-4" />
              {/if}
            {/if}
          </Button>
        {/if}
      </div>
    </div>
  </div>
{/if}
