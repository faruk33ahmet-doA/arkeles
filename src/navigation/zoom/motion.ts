/*
 * Hareket Anayasası — Anayasa madde 23.
 *
 * Bu dosya, izin verilen TÜM hareket parametrelerini tek yerde toplar.
 * Katman geçişleri yalnız buradaki değerleri kullanabilir. Bir bileşen
 * kendi süre/easing'ini tanımlarsa kod incelemesinde ret edilir (madde 23.4).
 *
 * Kurallar:
 *  - Yalnız transform + opacity animate edilir (madde 23.3).
 *  - Süre üst sınırı 300ms; katman geçişi 260ms (madde 23.4).
 *  - Hedef katman 1. karede etkileşime hazır (madde 23.5) — bu yüzden
 *    "animate" sırasında pointer-events KAPATILMAZ.
 *  - prefers-reduced-motion → süre 0 (madde 23.8, tokens.css'te de).
 */

import type { Transition, Variants } from "framer-motion";

const prefersReducedMotion =
  typeof window !== "undefined" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Katman zoom geçiş süresi (sn cinsinden — framer-motion birimi). */
export const LAYER_DURATION_S = prefersReducedMotion ? 0 : 0.26;

/** tokens.css --ease-out ile aynı eğri. */
export const EASE_OUT: [number, number, number, number] = [0.32, 0.72, 0, 1];

export const layerTransition: Transition = {
  duration: LAYER_DURATION_S,
  ease: EASE_OUT,
};

/*
 * Zoom "içeri" (kök → modül): gelen katman hafifçe büyüyerek ve netleşerek yaklaşır,
 * giden katman hafifçe küçülüp saydamlaşır. Mekânsal süreklilik hissi (madde 23.1).
 * Ölçek farkı küçük tutulur — gösterişli değil (madde 5).
 */
export const layerVariants: Variants = {
  // Modüle girerken: alttan/uzaktan gelir
  enterFromDeep: { opacity: 0, scale: 1.04 },
  // Modülden çıkarken: yukarı/geriye gider
  enterFromShallow: { opacity: 0, scale: 0.98 },
  center: { opacity: 1, scale: 1 },
  exitToDeep: { opacity: 0, scale: 0.98 },
  exitToShallow: { opacity: 0, scale: 1.04 },
};
