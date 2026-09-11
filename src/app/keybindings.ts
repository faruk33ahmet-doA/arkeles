import { useEffect } from "react";
import { useNavigationStore } from "@/state/navigationStore";
import { useCommandStore } from "@/state/commandStore";
import { useWorkStore } from "@/state/workStore";

/*
 * Global klavye bağlantıları — Anayasa madde 22.8, 27.5.
 *
 * 22.8  "Her gezinme klavyeyle geri alınabilir (Esc)."
 *
 * SPRINT 3: Esc artık DERİNDEN YÜZEYE doğru tek adım geri alır:
 *   not detayı → çalışma alanı → kurum listesi → katman (zoom out)
 *
 * Her basış TEK seviye geri gider; kullanıcı nerede olduğunu kaybetmez.
 */

export function useGlobalKeybindings(): void {
  const zoomOut = useNavigationStore((s) => s.zoomOut);
  const paletteOpen = useCommandStore((s) => s.open);
  const noteId = useWorkStore((s) => s.noteId);
  const workspaceId = useWorkStore((s) => s.workspaceId);
  const closeNote = useWorkStore((s) => s.closeNote);
  const closeWorkspace = useWorkStore((s) => s.closeWorkspace);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      // Palet açıkken Esc paleti kapatır (Radix yapar) — navigasyona karışmaz.
      if (paletteOpen) return;
      if (event.key !== "Escape") return;

      event.preventDefault();

      if (noteId) {
        closeNote();
        return;
      }
      if (workspaceId) {
        closeWorkspace();
        return;
      }
      zoomOut();
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [zoomOut, paletteOpen, noteId, workspaceId, closeNote, closeWorkspace]);
}
