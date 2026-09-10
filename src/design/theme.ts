/*
 * Tema yönetimi — Anayasa madde 17.1: tema seçimi vault'a YAZILMAZ.
 * Uygulama yapılandırmasıdır. Sprint 0'da yalnız runtime tutulur;
 * kalıcılık (Tauri app-config) Sprint 1'e bırakıldı.
 */

export type Theme = "dark" | "light";

/** Anayasa madde 29.2: karanlık tema varsayılan. */
export const DEFAULT_THEME: Theme = "dark";

export function applyTheme(theme: Theme): void {
  document.documentElement.setAttribute("data-theme", theme);
}
