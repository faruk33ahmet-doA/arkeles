import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import type { LifeScore } from "@/lib/generated";

/*
 * Hayat Skoru — Anayasa madde 11. En hassas bileşen.
 *
 * 11.1  Hermes HESAPLAR, vault'a yazar.
 * 11.2  ARKELÉS yalnız OKUR ve GÖSTERİR. Formülü bilmez.
 * 11.3  Tazelik belirtilir. Hermes kapalıyken skor kaybolmaz, BAYAT olur.
 *
 * Bu bileşende HİÇBİR hesaplama yoktur — bir toplama, bir ortalama, bir eşik
 * karşılaştırması bile yok. Renk bile skora göre değişmez, çünkü "72 iyi mi
 * kötü mü" bir YARGIDIR ve o yargı Hermes'in.
 *
 * Tek istisna: tazelik metni. O bir SAYIMDIR (kaç saat geçti), yargı değil
 * (madde 11.5).
 */

function formatFreshness(iso: string): string {
  const computed = new Date(iso).getTime();
  if (Number.isNaN(computed)) return "hesaplanma zamanı bilinmiyor";

  const hours = Math.floor((Date.now() - computed) / 3_600_000);
  if (hours < 1) return "az önce hesaplandı";
  if (hours < 24) return `${hours} saat önce hesaplandı`;
  return `${Math.floor(hours / 24)} gün önce hesaplandı`;
}

interface LifeScoreCardProps {
  score: LifeScore | null;
  loading?: boolean;
}

export function LifeScoreCard({ score, loading = false }: LifeScoreCardProps) {
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
      ) : loading ? null : (
        /* Hermes hiç hesaplamadıysa — hata değil, henüz yok (madde 26.2). */
        <EmptyState message="Henüz hesaplanmadı." />
      )}
    </Card>
  );
}
