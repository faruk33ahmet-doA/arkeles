import { create } from "zustand";

/*
 * Hermes aksiyon yüzeyi durumu — Sprint 4 madde 18.
 *
 * Cmd+K'dan bir aksiyon seçildiğinde ilgili yerde (çalışma alanı Genel
 * Bakış'ı veya Hermes katmanı) aksiyon yüzeyi açılır. Paralel bir komut
 * sistemi DEĞİL: aynı `ActionLauncher` bileşeni kullanılır.
 */

interface HermesState {
  /** Cmd+K'dan istenen aksiyon. Yüzey onu açtıktan sonra temizler. */
  pendingAction: string | null;
  pendingWorkspace: string | null;
  open: (actionId: string, workspaceId: string | null) => void;
  clear: () => void;
}

export const useHermesStore = create<HermesState>((set) => ({
  pendingAction: null,
  pendingWorkspace: null,
  open: (actionId, workspaceId) =>
    set({ pendingAction: actionId, pendingWorkspace: workspaceId }),
  clear: () => set({ pendingAction: null, pendingWorkspace: null }),
}));
