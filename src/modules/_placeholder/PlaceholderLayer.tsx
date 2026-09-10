import { LayerHost } from "@/navigation/layers/LayerHost";
import { EmptyState } from "@/ui/EmptyState";
import { useNavigationStore } from "@/state/navigationStore";
import { getLayer } from "@/navigation/layers/layerRegistry";

/*
 * PlaceholderLayer — Anayasa madde 26.4, 36.1, 36.2.
 *
 * 36.1  Dock BÜTÜN modülleri gösterir; zihinsel model baştan bütündür.
 * 26.4  Bu modüller "kırık değil, HENÜZ boş".
 * 26.2  Bu yüzden "Veri yok" / "Hata" / "Yapım aşamasında" gibi soğuk
 *       sistem dili kullanılmaz. Sakin ve dürüst bir cümle yeter.
 *
 * Tek bileşen 6 modüle hizmet eder — modül adı trail/store'dan okunur.
 * Amaç: 6 ayrı boş dosya yerine tek doğru boş durum (madde 39.2: modüler).
 */

export default function PlaceholderLayer() {
  const activeId = useNavigationStore((s) => s.trail[s.trail.length - 1]);
  const title = activeId ? getLayer(activeId).title : "";

  return (
    <LayerHost title={title}>
      <EmptyState
        message="Bu katman henüz boş."
        hint="Sırası geldiğinde burada gerçek veri olacak."
      />
    </LayerHost>
  );
}
