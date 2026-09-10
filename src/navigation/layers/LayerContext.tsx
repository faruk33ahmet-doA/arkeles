import { createContext, useContext } from "react";
import type { LayerDefinition } from "./types";

/*
 * LayerContext — bir katmanın KENDİ kimliğini bilmesi için.
 *
 * NEDEN VAR: Bir katman bileşeni "ben hangi katmanım?" sorusunu global
 * navigasyon durumundan okuyamaz. Çünkü zoom geçişi sırasında iki katman
 * kısa süre birlikte DOM'da bulunur (madde 23.7: kaynak katman geçiş
 * bitince unmount edilir) ve global durum ARTIK YENİ katmanı gösterir.
 * Çıkmakta olan katman global durumu okursa yanlış başlığı gösterir.
 *
 * Bu yüzden kimlik ZoomEngine tarafından mount anında sabitlenir ve
 * context ile aşağı verilir. Çıkan katman kendi değerini korur.
 *
 * Bu, madde 35.2'nin ("sıcak veri Context'te tutulmaz") istisnası değildir:
 * buradaki değer katmanın ömrü boyunca DEĞİŞMEZ, dolayısıyla render
 * fırtınası üretmez.
 */

const LayerContext = createContext<LayerDefinition | null>(null);

export const LayerProvider = LayerContext.Provider;

/** Aktif olarak render edilen katmanın tanımı. */
export function useCurrentLayer(): LayerDefinition {
  const layer = useContext(LayerContext);
  if (!layer) {
    throw new Error("useCurrentLayer yalnız bir katman içinde kullanılabilir");
  }
  return layer;
}
