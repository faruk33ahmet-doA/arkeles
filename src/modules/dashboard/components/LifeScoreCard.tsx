import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import type { LifeScore } from "@/modules/today/fixtures";

/*
 * Hayat Skoru — Anayasa madde 11. En hassas bileşen.
 *
 * 11.1  Hermes HESAPLAR, vault'a yazar.
 * 11.2  ARKELÉS yalnız OKUR ve GÖSTERİR. Formülü bilmez.
 * 11.3  Tazelik belirtilir. Hermes kapalıyken skor kaybolmaz, BAYAT olur.
 *
 * Bu yüzden bu bileşende HİÇBİR hesaplama yoktur — bir toplama, bir
 * ortalama, bir eşik karşılaştırması bile yok. Renk bile skora göre
 * değişmez, çünkü "72 iyi mi kötü mü" bir YARGIDIR ve o yargı Hermes'in.
 */

function formatFreshness(iso: string): string {
  const computed = new Date(iso);
  const hours = Math.floor((Date.now() - computed.getTime()) / 3_600_000);
  if (hours < 1) return "az önce hesaplandı";
  if (hours < 24) return `${hours} saat önce hesaplandı`;
  const days = Math.floor(hours / 24);
  return `${days} gün önce hesaplandı`;
}

export function LifeScoreCard({ score }: { score: LifeScore | null }) {
  return (
    <Card title="Hayat skoru">
      {score ? (
        <div className="flex flex-col gap-1">
          <span className="text-2xl font-semibold tabular-nums text-text-primary">
            {score.value}
          </span>
          {/* Madde 11.3: tazelik zorunlu. */}
          <span className="text-xs text-text-tertiary">
            {formatFreshness(score.computedAt)}
          </span>
        </div>
      ) : (
        /* Hermes hiç hesaplamadıysa — hata değil, henüz yok (madde 26.2). */
        <EmptyState message="Henüz hesaplanmadı." />
      )}
    </Card>
  );
}
