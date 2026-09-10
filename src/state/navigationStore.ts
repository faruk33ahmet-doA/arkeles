import { create } from "zustand";
import type { LayerId } from "@/navigation/layers/types";
import { ROOT_LAYER_ID } from "@/navigation/layers/layerRegistry";

/*
 * Navigasyon durumu — Anayasa madde 22, 35.2.
 *
 * Sıcak veri değil, saf UI durumu. Zustand seçilir çünkü selector'lı abonelik
 * ile yalnız ilgili component render olur (madde 35.2). Context kullanılmaz.
 *
 * "trail" = Zoom Trail (madde 22.3). Kullanıcının nereden geldiğinin kaydı.
 * Kök her zaman dizinin ilk elemanıdır ve çıkarılamaz.
 */

interface NavigationState {
  /** Zoom Trail — kökten aktif katmana kadarki yol. En az 1 eleman. */
  trail: LayerId[];
  /** Bir zoom geçişi sürüyor mu? Yalnız gösterge amaçlı (madde 23.5: bloke etmez). */
  isTransitioning: boolean;

  /** Aktif (en üstteki) katman. */
  activeLayer: () => LayerId;

  /** Bir modüle zoom ile gir. Kök→modül tek adımdır (madde 22.5). */
  zoomTo: (layer: LayerId) => void;
  /** Bir adım geri (Esc ile de tetiklenir — madde 22.8). */
  zoomOut: () => void;
  /** Doğrudan köke dön. */
  zoomToRoot: () => void;
  setTransitioning: (value: boolean) => void;
}

export const useNavigationStore = create<NavigationState>((set, get) => ({
  trail: [ROOT_LAYER_ID],
  isTransitioning: false,

  activeLayer: () => {
    const { trail } = get();
    return trail[trail.length - 1] ?? ROOT_LAYER_ID;
  },

  zoomTo: (layer) =>
    set((state) => {
      if (state.trail[state.trail.length - 1] === layer) return state;
      // Kök kendisinin çocuğu olamaz: köke "zoom" etmek köke DÖNMEKTİR.
      if (layer === ROOT_LAYER_ID) return { trail: [ROOT_LAYER_ID] };
      // Modüle geçişte trail = [kök, modül] — madde 22.5: en fazla 2 seviye.
      return { trail: [ROOT_LAYER_ID, layer] };
    }),

  zoomOut: () =>
    set((state) => {
      if (state.trail.length <= 1) return state;
      return { trail: state.trail.slice(0, -1) };
    }),

  zoomToRoot: () => set({ trail: [ROOT_LAYER_ID] }),

  setTransitioning: (value) => set({ isTransitioning: value }),
}));
