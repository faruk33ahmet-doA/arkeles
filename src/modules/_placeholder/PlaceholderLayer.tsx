import { LayerHost } from "@/navigation/layers/LayerHost";
import { useCurrentLayer } from "@/navigation/layers/LayerContext";
import { EmptyState } from "@/ui/EmptyState";

/*
 * PlaceholderLayer — Anayasa madde 26.4, 36.1, 36.2.
 *
 * 36.1  Dock BÜTÜN modülleri gösterir; zihinsel model baştan bütündür.
 * 26.4  Bu modüller "kırık değil, HENÜZ boş".
 * 26.2  Bu yüzden "Veri yok" / "Hata" / "Yapım aşamasında" gibi soğuk
 *       sistem dili kullanılmaz. Sakin ve dürüst bir cümle yeter.
 *
 * Tek bileşen 6 modüle hizmet eder. Başlığını LayerContext'ten alır,
 * global navigasyon durumundan DEĞİL — geçiş sırasında çıkan katman
 * yeni katmanın başlığını gösterirdi (bkz. LayerContext yorumu).
 */

export default function PlaceholderLayer() {
  const layer = useCurrentLayer();

  return (
    <LayerHost title={layer.title}>
      <EmptyState
        message="Bu katman henüz boş."
        hint="Sırası geldiğinde burada gerçek veri olacak."
      />
    </LayerHost>
  );
}
