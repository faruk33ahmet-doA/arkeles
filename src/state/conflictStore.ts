import { create } from "zustand";

/*
 * Çakışma durumu — Anayasa madde 20.3, 25.4.
 *
 * 25.4  "Yalnızca VERİ KAYBI RİSKİ taşıyan durum kullanıcıyı bloke edebilir:
 *        yazma çakışması. Bunun dışında hiçbir şey akışı kesmez."
 *
 * Yani bu, uygulamada akışı kesmeye YETKİLİ tek durumdur. Bu yüzden ayrı
 * bir store: bir tane vardır, global tektir, ve kapanana kadar durur.
 */

export interface ConflictInfo {
  /** Kullanıcının ne yapmaya çalıştığı — sakin bir cümle. */
  action: string;
  /** Çakışan notun kimliği; "güncel halini göster" için gerekli. */
  noteId: string | null;
  /** Yeniden denemek için: aynı mutasyonu tazelenmiş guard ile çalıştırır. */
  retry: (() => void) | null;
}

interface ConflictState {
  conflict: ConflictInfo | null;
  open: (info: ConflictInfo) => void;
  dismiss: () => void;
}

export const useConflictStore = create<ConflictState>((set) => ({
  conflict: null,
  open: (info) => set({ conflict: info }),
  dismiss: () => set({ conflict: null }),
}));
