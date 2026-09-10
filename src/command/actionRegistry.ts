import type { LayerId } from "@/navigation/layers/types";
import { listLayers } from "@/navigation/layers/layerRegistry";

/*
 * Aksiyon Kaydı — Anayasa madde 27.3:
 * "Bütün aksiyonlar merkezi bir aksiyon kaydından tanımlanır.
 *  Komut paleti bu kaydı okur. Kayıtta olmayan aksiyon yoktur."
 *
 * Bu kayıt Sprint 0'da kurulur çünkü sonradan kurulan komut paletleri
 * hep eksik kalır (bkz. teknik değerlendirme, A7 gerekçesi).
 *
 * Aksiyon türleri anayasadaki yetki ayrımını (madde 8) yansıtır:
 *   "navigate"  → katman değiştirir, yazma yok
 *   "mechanic"  → mekanik mutasyon, ARKELÉS yazar (madde 8.1) — Sprint 2
 *   "semantic"  → semantik mutasyon, Hermes yapar (madde 8.2) — Sprint 4
 *   "system"    → uygulama yapılandırması (madde 17.2)
 *
 * Anayasa madde 18.2: Hermes'in bildirmediği yetenek arayüzde HİÇ görünmez.
 * Bu yüzden "semantic" aksiyonlar bir `capability` anahtarı taşır ve
 * palet, Hermes'in yetenek listesinde olmayanları filtreler.
 */

export type ActionKind = "navigate" | "mechanic" | "semantic" | "system";

export interface Action {
  id: string;
  /** Palette görünen ad. */
  title: string;
  /** Gruplama başlığı. */
  group: string;
  kind: ActionKind;
  /** Arama eşleşmesini genişletir (Türkçe/İngilizce eş anlam). */
  keywords?: string[];
  /** Yalnız "semantic" için: Hermes yetenek anahtarı (madde 18.2). */
  capability?: string;
  /** Aksiyonun kendisi. Sprint 0'da navigate dışındakiler tanımsız. */
  run?: (ctx: ActionContext) => void | Promise<void>;
}

export interface ActionContext {
  enterLayer: (layer: LayerId) => void;
  closePalette: () => void;
}

/** Katmanlardan otomatik türeyen navigasyon aksiyonları (madde 27.4). */
function navigationActions(): Action[] {
  return listLayers().map((layer) => ({
    id: `navigate:${layer.id}`,
    title: layer.title,
    group: "Git",
    kind: "navigate" as const,
    keywords: [layer.id],
    run: (ctx) => {
      ctx.enterLayer(layer.id);
      ctx.closePalette();
    },
  }));
}

/*
 * Sprint 0'da yalnız navigasyon aksiyonları GERÇEKTİR.
 *
 * Hızlı Yakalama (madde 10) ve semantik aksiyonlar burada BİLİNÇLİ OLARAK
 * TANIMSIZDIR — çünkü madde 18.2 "tıklandığında hata veren buton yoktur"
 * diyor. Yazma altyapısı gelmeden (Sprint 2) paleti sahte komutla doldurmak
 * bu maddeyi ihlal ederdi.
 */
export function getActions(): Action[] {
  return [...navigationActions()];
}
