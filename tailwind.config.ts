import type { Config } from "tailwindcss";

/*
 * Anayasa madde 29 & 31.3:
 * Tailwind yalnız token'lara köprüdür. Renk/spacing/radius değerleri burada
 * DEĞİL, src/design/tokens.css içinde yaşar. Bu config o değişkenleri
 * utility sınıflarına bağlar — kaynak tek yerde kalır.
 */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["selector", '[data-theme="dark"]'],
  theme: {
    // Varsayılan Tailwind ölçeklerini SİLİYORUZ. Yalnız anayasa ölçeği geçerli.
    spacing: {
      0: "0",
      1: "var(--space-1)",
      2: "var(--space-2)",
      3: "var(--space-3)",
      4: "var(--space-4)",
      6: "var(--space-6)",
      8: "var(--space-8)",
      12: "var(--space-12)",
      16: "var(--space-16)",
    },
    borderRadius: {
      none: "0",
      sm: "var(--radius-sm)",
      md: "var(--radius-md)",
      lg: "var(--radius-lg)",
      full: "9999px",
    },
    fontSize: {
      xs: ["var(--text-xs)", { lineHeight: "var(--leading-normal)" }],
      sm: ["var(--text-sm)", { lineHeight: "var(--leading-normal)" }],
      base: ["var(--text-base)", { lineHeight: "var(--leading-normal)" }],
      lg: ["var(--text-lg)", { lineHeight: "var(--leading-tight)" }],
      xl: ["var(--text-xl)", { lineHeight: "var(--leading-tight)" }],
      "2xl": ["var(--text-2xl)", { lineHeight: "var(--leading-tight)" }],
    },
    fontWeight: {
      regular: "var(--weight-regular)",
      medium: "var(--weight-medium)",
      semibold: "var(--weight-semibold)",
    },
    fontFamily: {
      sans: "var(--font-family)",
    },
    boxShadow: {
      none: "none",
      1: "var(--elevation-1)",
      2: "var(--elevation-2)",
      3: "var(--elevation-3)",
    },
    transitionTimingFunction: {
      out: "var(--ease-out)",
      "in-out": "var(--ease-in-out)",
    },
    transitionDuration: {
      fast: "var(--dur-fast)",
      base: "var(--dur-base)",
      layer: "var(--dur-layer)",
    },
    zIndex: {
      base: "var(--z-base)",
      layer: "var(--z-layer)",
      panel: "var(--z-panel)",
      trail: "var(--z-trail)",
      dock: "var(--z-dock)",
      command: "var(--z-command)",
      conflict: "var(--z-conflict)",
    },
    backdropBlur: {
      glass: "var(--blur-glass)",
      "glass-strong": "var(--blur-glass-strong)",
    },
    colors: {
      transparent: "transparent",
      current: "currentColor",
      surface: {
        0: "var(--surface-0)",
        1: "var(--surface-1)",
        2: "var(--surface-2)",
        3: "var(--surface-3)",
      },
      text: {
        primary: "var(--text-primary)",
        secondary: "var(--text-secondary)",
        tertiary: "var(--text-tertiary)",
        "on-accent": "var(--text-on-accent)",
      },
      border: {
        subtle: "var(--border-subtle)",
        DEFAULT: "var(--border-default)",
        strong: "var(--border-strong)",
      },
      accent: {
        DEFAULT: "var(--accent)",
        muted: "var(--accent-muted)",
      },
      status: {
        online: "var(--status-online)",
        offline: "var(--status-offline)",
        attention: "var(--status-attention)",
      },
      glass: {
        bg: "var(--glass-bg)",
        border: "var(--glass-border)",
      },
    },
    extend: {},
  },
  plugins: [],
} satisfies Config;
