# ETLauncher Design Language

## Overview

A Minecraft launcher with a bold, gamer aesthetic. Pixelated fonts, sharp edges, glowing accents, and chunky UI elements. Dark theme only.

---

## Core Principles

1. **Pixelated** - Pixel fonts, sharp 0px corners, blocky shapes
2. **Bold** - Thick 2px borders, uppercase text, wide letter spacing
3. **Glowing** - Accent elements have glow effects
4. **Dark** - Deep dark backgrounds, high contrast

---

## Typography

### Fonts

```css
--font-pixel: 'Silkscreen', monospace;      /* Body text */
--font-pixel-bold: 'Press Start 2P', monospace;  /* Special headers */
```

### Style

- All text: UPPERCASE
- Letter spacing: 0.05em - 0.08em (wide)
- No font smoothing (crisp pixels)
- Headings get text shadows for depth

### Scale

| Use | Size | Style |
|-----|------|-------|
| Page title | 20-24px | Uppercase, bold, text-shadow |
| Section title | 14px | Uppercase, wide tracking |
| Body | 12-14px | Uppercase |
| Small/Caption | 10-12px | Uppercase, muted |

---

## Colors

### Primary Accent: Vibrant Oxidized Copper

```css
--primary: oklch(0.72 0.18 172);  /* Bright cyan-teal */
```

High saturation for glow effects.

### Background

```css
--background: oklch(0.12 0.015 280);  /* Deep dark blue-black */
--card: oklch(0.16 0.015 280);        /* Slightly elevated */
--sidebar: oklch(0.08 0.015 280);     /* Darkest */
```

### Borders

```css
--border: oklch(0.30 0.02 280);  /* Visible, chunky */
```

2px borders throughout.

### Gaming Palette

```css
--chart-1: oklch(0.72 0.18 172);  /* Cyan/teal (primary) */
--chart-2: oklch(0.65 0.25 280);  /* Purple */
--chart-3: oklch(0.80 0.22 85);   /* Gold */
--chart-4: oklch(0.65 0.28 330);  /* Pink */
--chart-5: oklch(0.70 0.22 145);  /* Green */
```

---

## Border Radius

```css
--radius: 0px;  /* Sharp corners everywhere */
```

No rounded corners. Everything is blocky/pixelated.

---

## Component Specifications

### Buttons

- Sharp 0px corners
- 2px solid border
- Uppercase text, wide letter spacing
- Hover: translateY(-1px) lift effect
- Active: translateY(1px) press effect
- Primary buttons: Glow effect with box-shadow

```css
.bg-primary {
  box-shadow:
    0 0 10px oklch(0.72 0.18 172 / 0.4),
    0 0 20px oklch(0.72 0.18 172 / 0.2);
}
```

### Cards

- 2px solid border
- No rounded corners
- Hover: Border glows with primary color

### Inputs

- 2px solid border
- No rounded corners
- Focus: Primary color border + glow

### Sidebar

- Darkest background
- Menu items: 2px border on active state
- Active: Primary background + glow effect
- Uppercase labels, wide letter spacing

### Titlebar

- Custom window controls
- 2px bottom border
- Pixel font title
- Primary color logo box

---

## Effects

### Glow

Primary elements get a glow effect:

```css
box-shadow:
  0 0 10px oklch(0.72 0.18 172 / 0.4),
  0 0 20px oklch(0.72 0.18 172 / 0.2);
```

Enhanced on hover:

```css
box-shadow:
  0 0 15px oklch(0.72 0.18 172 / 0.6),
  0 0 30px oklch(0.72 0.18 172 / 0.3);
```

### Text Shadow

Headings:

```css
text-shadow: 2px 2px 0 oklch(0.72 0.18 172 / 0.3);
```

### Button Interaction

- Hover: translateY(-1px)
- Active: translateY(1px)
- Transition: 0.1s ease-out

---

## Scrollbars

```css
::-webkit-scrollbar {
  width: 12px;
}

::-webkit-scrollbar-thumb {
  background: var(--muted);
  border: 2px solid var(--border);
}
```

Chunky, visible scrollbars with 2px borders.

---

## Layout

### App Shell

```
+------------------------------------------+
|  [ET] ETLAUNCHER           [_] [□] [X]   |  <- Titlebar (36px)
+--------+---------------------------------+
|        |                                 |
| SIDE   |         MAIN CONTENT            |
| BAR    |                                 |
|        |                                 |
+--------+---------------------------------+
```

### Spacing

```
4px  - Tight gaps
8px  - Default gap
12px - Between sections
16px - Component padding
24px - Major sections
```

---

## Animation

Keep animations snappy and minimal:

- Duration: 100-150ms
- Easing: ease-out
- No smooth transitions (feels more "digital")
