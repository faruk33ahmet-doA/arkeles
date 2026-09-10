import { Suspense, lazy, useMemo } from "react";
import { AnimatePresence, LazyMotion, domAnimation, m } from "framer-motion";
import { useNavigationStore } from "@/state/navigationStore";
import { getLayer } from "@/navigation/layers/layerRegistry";
import { layerTransition, layerVariants } from "./motion";

/*
 * Zoom Engine — Anayasa madde 22.4, 22.5, 23.
 *
 * Sorumluluğu TEK: aktif katmanı mount etmek ve katmanlar arası geçişi
 * mekânsal süreklilikle sunmak. Kendi başına iş mantığı taşımaz.
 *
 * Anayasa uyumu:
 *  - LazyMotion + domAnimation → framer-motion runtime'ı ~%60 küçülür (madde 14, 34.1)
 *  - Yalnız 1 katman DOM'da: AnimatePresence mode="popLayout" + tek child.
 *    Kaynak katman geçiş biter bitmez unmount olur (madde 23.7).
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
  const setTransitioning = useNavigationStore((s) => s.setTransitioning);

  const activeId = trail[trail.length - 1]!;
  const layer = getLayer(activeId);
  const isModule = layer.level === 1;

  const LayerComponent = useMemo(
    () => getLayerComponent(layer.id, layer.component),
    [layer.id, layer.component],
  );

  return (
    <LazyMotion features={domAnimation} strict>
      <div className="relative h-full w-full overflow-hidden">
        <AnimatePresence
          mode="popLayout"
          initial={false}
          onExitComplete={() => setTransitioning(false)}
        >
          <m.div
            key={activeId}
            className="absolute inset-0 z-layer"
            variants={layerVariants}
            initial={isModule ? "enterFromDeep" : "enterFromShallow"}
            animate="center"
            exit={isModule ? "exitToShallow" : "exitToDeep"}
            transition={layerTransition}
            onAnimationStart={() => setTransitioning(true)}
            onAnimationComplete={() => setTransitioning(false)}
          >
            {/* Suspense fallback: boş — spinner yok (madde 23.6, 28).
                Sprint 1'de veri animasyondan önce ısıtılacağı için pratikte görünmez. */}
            <Suspense fallback={<div className="h-full w-full bg-surface-1" />}>
              <LayerComponent />
            </Suspense>
          </m.div>
        </AnimatePresence>
      </div>
    </LazyMotion>
  );
}
