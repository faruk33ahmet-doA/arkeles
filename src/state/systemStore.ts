import { create } from "zustand";
import { DEFAULT_THEME, applyTheme, type Theme } from "@/design/theme";

/*
 * Sistem/kabuk durumu — Anayasa madde 17.2: uygulama yapılandırması, bilgi değil.
 * Vault'a yazılmaz. Sprint 0'da runtime; kalıcılık Sprint 1 (Tauri app-config).
 */

interface SystemState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export const useSystemStore = create<SystemState>((set) => ({
  theme: DEFAULT_THEME,
  setTheme: (theme) => {
    applyTheme(theme);
    set({ theme });
  },
}));
