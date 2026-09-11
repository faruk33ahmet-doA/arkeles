import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

/*
 * Sanallaştırılmış liste — Anayasa madde 35.3, Sprint 3 madde 14.
 *
 * 35.3  "200+ satırlı listeler sanallaştırılır."
 *
 * Eşiğin ALTINDA sanallaştırma YAPILMAZ: 12 satır için sanal pencere kurmak
 * gereksiz iştir (madde 35.1) ve kaydırma davranışını bozar. Bu bileşen
 * eşiği kendi kontrol eder, çağıran düşünmek zorunda kalmaz.
 *
 * Görevler, belgeler ve notlar AYNI bileşeni kullanır — Sprint 3 madde 14
 * "önce gerçek tekrar oluşsun, sonra abstraction yap" der; tekrar üç yerde
 * oluştu, soyutlama burada.
 */

const VIRTUALIZE_THRESHOLD = 200;

interface VirtualListProps<T> {
  items: T[];
  /** Satır yüksekliği tahmini (px). Ölçüm sonrası düzeltilir. */
  estimateSize?: number;
  /** Kaydırma alanının en fazla yüksekliği. */
  maxHeight?: number;
  keyOf: (item: T, index: number) => string;
  renderItem: (item: T) => React.ReactNode;
}

export function VirtualList<T>({
  items,
  estimateSize = 40,
  maxHeight = 560,
  keyOf,
  renderItem,
}: VirtualListProps<T>) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => estimateSize,
    overscan: 8,
  });

  // Küçük listeler düz render edilir.
  if (items.length < VIRTUALIZE_THRESHOLD) {
    return (
      <div className="divide-y divide-border-subtle">
        {items.map((item, index) => (
          <div key={keyOf(item, index)}>{renderItem(item)}</div>
        ))}
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      style={{ maxHeight }}
      className="overflow-y-auto"
      data-virtualized="true"
    >
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((row) => {
          const item = items[row.index]!;
          return (
            <div
              key={keyOf(item, row.index)}
              ref={virtualizer.measureElement}
              data-index={row.index}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${row.start}px)`,
              }}
              className="border-b border-border-subtle"
            >
              {renderItem(item)}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/** Test ve ölçüm için: eşik değeri tek kaynakta. */
export const VIRTUALIZATION_THRESHOLD = VIRTUALIZE_THRESHOLD;
