import { Suspense, lazy, useMemo } from "react";
import { AnimatePresence, LazyMotion, domAnimation, m } from "framer-motion";
import { useNavigationStore } from "@/state/navigationStore";
import { getLayer } from "@/navigation/layers/layerRegistry";
import { LayerProvider } from "@/navigation/layers/LayerContext";
import { layerTransition, layerVariants, type ZoomDirection } from "./motion";

/*
 * Zoom Engine — Anayasa madde 22.4, 22.5, 23.
 *
 * Sorumluluğu TEK: aktif katmanı mount etmek ve katmanlar arası geçişi
 * mekânsal süreklilikle sunmak. Kendi başına iş mantığı taşımaz.
 *
 * Anayasa uyumu:
 *  - LazyMotion + domAnimation → framer-motion runtime'ı ~%60 küçülür (madde 14, 34.1)
 *  - Tek child + değişen key: AnimatePresence geçiş bitince kaynak katmanı
 *    unmount eder (madde 23.7). Geçiş sırasında kısa süre iki katman DOM'dadır.
 *  - Animasyon dekorasyon: gelen katmana pointer-events engeli KONMAZ (madde 23.5).
 *  - Süre/easing yalnız motion.ts'ten (madde 23.4).
 *
 * Sprint 0 kapsamı: altyapı kurulu, geçiş çalışır. "Prefetch'li giriş" ve
 * seviye-2 panel geçişleri (madde 22.6) Sprint 1+.
 */

// Katman bileşenlerini lazy sarmalıyoruz — kod bölme, ilk yük bütçesi (madde 34.1).
const layerComponentCache = new Map<string, React.LazyExoticComponent<React.ComponentType>>();

function getLayerComponent(id: string, loader: () => Promise<{ default: React.ComponentType }>) {
  let cached = layerComponentCache.get(id);
  if (!cached) {
    cached = lazy(loader);
    layerComponentCache.set(id, cached);
  }
  return cached;
}

export function ZoomEngine() {
  const trail = useNavigationStore((s) => s.trail);

  const activeId = trail[trail.length - 1]!;
  const layer = getLayer(activeId);

  /*
   * Kameranın yönü hedef katmanın seviyesinden çıkar:
   * modüle gidiyorsak "in", panele dönüyorsak "out" (bkz. motion.ts).
   */
  const direction: ZoomDirection = layer.level === 1 ? "in" : "out";

  const LayerComponent = useMemo(
    () => getLayerComponent(layer.id, layer.component),
    [layer.id, layer.component],
  );

  return (
    <LazyMotion features={domAnimation} strict>
      <div className="relative h-full w-full overflow-hidden">
        {/*
          Varsayılan "sync" mod: iki katman aynı anda animate olur, arada
          boşluk kalmaz (madde 23.5). mode="wait" gecikme yaratır, "popLayout"
          ise bizim absolute katmanlarımızda boşa layout ölçümü yapar.

          `custom={direction}`: ÇIKAN katman önceki render'dan klonlandığı için
          kendi çıkış yönünü bilemez. AnimatePresence bu değeri çıkış anında
          varyant fonksiyonuna taşır — yön böylece doğru olur.
        */}
        <AnimatePresence initial={false} custom={direction}>
          <m.div
            key={activeId}
            custom={direction}
            className="absolute inset-0 z-layer"
            variants={layerVariants}
            initial="enter"
            animate="center"
            exit="exit"
            transition={layerTransition}
          >
            {/* Katman kimliği mount anında sabitlenir — çıkan katman
                global durumu okuyup yanlış başlık göstermesin (LayerContext). */}
            <LayerProvider value={layer}>
              {/* Suspense fallback: boş — spinner yok (madde 23.6, 28).
                  Sprint 1'de veri animasyondan önce ısıtılacağı için pratikte görünmez. */}
              <Suspense fallback={<div className="h-full w-full bg-surface-1" />}>
                <LayerComponent />
              </Suspense>
            </LayerProvider>
          </m.div>
        </AnimatePresence>
      </div>
    </LazyMotion>
  );
}
