import { create } from "zustand";

/*
 * Komut paleti aç/kapa durumu — Anayasa madde 27.
 * İçerik ve aksiyonlar actionRegistry'de; bu store yalnız görünürlük.
 */

interface CommandState {
  open: boolean;
  setOpen: (open: boolean) => void;
  toggle: () => void;
}

export const useCommandStore = create<CommandState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
  toggle: () => set((s) => ({ open: !s.open })),
}));
