import { useEffect } from "react";
import { useNavigationStore } from "@/state/navigationStore";
import { useCommandStore } from "@/state/commandStore";

/*
 * Global klavye bağlantıları — Anayasa madde 22.8, 27.5.
 *
 * 22.8  "Her gezinme klavyeyle geri alınabilir (Esc)."
 * 27.5  "Fareyle yapılabilen her ana işlem klavyeyle de yapılabilir."
 *
 * Cmd+K burada DEĞİL, useCommandPalette.ts'te — palet kendi kısayolunun
 * sahibi. Bu dosya navigasyon kısayollarından sorumlu.
 */

export function useGlobalKeybindings(): void {
  const zoomOut = useNavigationStore((s) => s.zoomOut);
  const paletteOpen = useCommandStore((s) => s.open);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      // Palet açıkken Esc paleti kapatır (Radix yapar) — navigasyona karışmaz.
      if (paletteOpen) return;

      if (event.key === "Escape") {
        event.preventDefault();
        zoomOut();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [zoomOut, paletteOpen]);
}
