import { create } from "zustand";

/*
 * İş modülü gezinme durumu — Sprint 3 madde 12.
 *
 * KRİTİK KURAL: alt paneller Zoom Trail'e YENİ SEVİYE EKLEMEZ.
 *
 *   ARKELÉS • İş • WIF      ← Görevler'e geçince AYNI KALIR
 *
 * Sebep anayasa madde 22.5 (en fazla 2 zoom seviyesi) ve 22.6 (alt
 * katmanlarda panel geçişi). Çalışma alanı trail'de GÖRÜNÜR ama bir zoom
 * seviyesi DEĞİLDİR — bu yüzden `navigationStore.trail`'e dokunmaz, burada
 * ayrı tutulur.
 */

/** Sprint 3 madde 4 panelleri. */
export type WorkPanel =
  | "overview"
  | "tasks"
  | "projects"
  | "documents"
  | "meetings"
  | "notes";

export const WORK_PANELS: { id: WorkPanel; label: string }[] = [
  { id: "overview", label: "Genel Bakış" },
  { id: "tasks", label: "Görevler" },
  { id: "projects", label: "Projeler" },
  { id: "documents", label: "Belgeler" },
  { id: "meetings", label: "Toplantılar" },
  { id: "notes", label: "Notlar" },
];

interface WorkState {
  /** Açık çalışma alanı. `null` → kurum listesi görünür. */
  workspaceId: string | null;
  /** Aktif panel. Çalışma alanı değişince Genel Bakış'a döner. */
  panel: WorkPanel;
  /** Açık not detayı. Panelin ÜSTÜNDE bir alt yüzeydir. */
  noteId: string | null;
  /**
   * Not detayında seçili görev — Sprint 5 madde 8. Sürükle-bırakın klavye
   * karşılığı (⌥↑/⌥↓ ve komut paleti) bu görevi taşır.
   */
  selectedTaskId: string | null;

  openWorkspace: (id: string) => void;
  closeWorkspace: () => void;
  setPanel: (panel: WorkPanel) => void;
  openNote: (noteId: string) => void;
  closeNote: () => void;
  selectTask: (taskId: string | null) => void;
}

export const useWorkStore = create<WorkState>((set) => ({
  workspaceId: null,
  panel: "overview",
  noteId: null,
  selectedTaskId: null,

  openWorkspace: (id) =>
    set((state) =>
      state.workspaceId === id
        ? state
        : // Yeni kuruma girerken önceki panelin/notun taşınması kafa
          // karıştırırdı: her kurum Genel Bakış'tan başlar (madde 3).
          { workspaceId: id, panel: "overview", noteId: null, selectedTaskId: null },
    ),

  closeWorkspace: () =>
    set({ workspaceId: null, panel: "overview", noteId: null, selectedTaskId: null }),

  setPanel: (panel) =>
    set((state) =>
      state.panel === panel ? state : { panel, noteId: null, selectedTaskId: null },
    ),

  openNote: (noteId) => set({ noteId, selectedTaskId: null }),
  closeNote: () => set({ noteId: null, selectedTaskId: null }),
  selectTask: (selectedTaskId) => set({ selectedTaskId }),
}));
