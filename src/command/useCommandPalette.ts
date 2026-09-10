import { useEffect } from "react";
import { useCommandStore } from "@/state/commandStore";

/*
 * Cmd+K kısayolu — Anayasa madde 27.1.
 *
 * Global klavye dinleyicisi tek yerde. Bileşenlerin kendi
 * window.addEventListener'ı olması yasak (madde 35.1: gereksiz iş).
 */

export function useCommandPaletteHotkey(): void {
  const toggle = useCommandStore((s) => s.toggle);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      // macOS: Cmd+K · diğer: Ctrl+K
      if (event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        toggle();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggle]);
}
