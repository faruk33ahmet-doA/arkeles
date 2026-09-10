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
 *
 * ─────────────────────────────────────────────────────────────────────────
 * MEKÂNSAL MODEL — madde 23.1: "hareketin tek meşru işi mekânsal süreklilik"
 *
 * Panel (seviye 0) DIŞ/GENİŞ görünümdür. Modül (seviye 1) DAHA DERİNdedir.
 * Kamera bir eksen üzerinde ileri-geri gider:
 *
 *   İÇERİ (panel → modül) — kamera İLERİ:
 *     çıkan panel   : büyüyerek solar   (kamera içinden geçer)  1 → 1.04
 *     gelen modül   : küçükten yerine oturur (ileriden yaklaşır) 0.98 → 1
 *
 *   DIŞARI (modül → panel) — kamera GERİ:
 *     çıkan modül   : küçülerek solar   (ileride uzaklaşır)      1 → 0.98
 *     gelen panel   : büyükten yerine oturur (içindeydik)        1.04 → 1
 *
 * Ölçek farkı bilinçli olarak KÜÇÜK (%4). Gösterişli değil (madde 5),
 * ama yön hissi net.
 * ─────────────────────────────────────────────────────────────────────────
 */

import type { Transition, Variants } from "framer-motion";

const prefersReducedMotion =
  typeof window !== "undefined" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Katman zoom geçiş süresi (sn — framer-motion birimi). */
export const LAYER_DURATION_S = prefersReducedMotion ? 0 : 0.26;

/** tokens.css --ease-out ile aynı eğri. */
export const EASE_OUT: [number, number, number, number] = [0.32, 0.72, 0, 1];

export const layerTransition: Transition = {
  duration: LAYER_DURATION_S,
  ease: EASE_OUT,
};

/** Kameranın gidiş yönü. */
export type ZoomDirection = "in" | "out";

/** Uzaktaki/derindeki yüzeyin ölçeği. */
const FAR = 0.98;
/** Yakındaki/geçilen yüzeyin ölçeği. */
const NEAR = 1.04;

/*
 * Varyantlar FONKSİYONDUR çünkü çıkış yönü, o katman mount edildiğinde
 * henüz BİLİNMEZ. Framer Motion'ın `custom` mekanizması yönü çıkış anında
 * AnimatePresence'tan varyanta taşır — yön bilgisi bu yüzden burada
 * parametre olarak alınır, prop olarak sabitlenmez.
 */
export const layerVariants: Variants = {
  enter: (direction: ZoomDirection) =>
    direction === "in"
      ? { opacity: 0, scale: FAR }   // modül ileriden yaklaşır
      : { opacity: 0, scale: NEAR }, // panel içindeydik, yerine oturur

  center: { opacity: 1, scale: 1 },

  exit: (direction: ZoomDirection) =>
    direction === "in"
      ? { opacity: 0, scale: NEAR }  // panel büyüyerek geçilir
      : { opacity: 0, scale: FAR },  // modül ileride uzaklaşır
};
