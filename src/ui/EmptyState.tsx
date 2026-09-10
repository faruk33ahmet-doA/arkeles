/*
 * EmptyState — Anayasa madde 26 (Boş Durum Doktrini).
 *
 * 26.2: Boş durum bir HATA DEĞİLDİR. "Veri yok", "Hata", "Bulunamadı"
 *       gibi soğuk metin kullanılmaz.
 * 26.3: Sakin, kısa, varsa tek bir sonraki adım.
 *
 * Bu bileşen Sprint 0'da yazılır çünkü madde 26.1: boş durum, dolu durumla
 * AYNI SPRINTTE tasarlanır.
 */

interface EmptyStateProps {
  /** Kısa, sakin cümle. Soğuk sistem dili kullanılmaz. */
  message: string;
  /** Varsa tek bir sonraki adım — birden fazla değil (madde 26.3). */
  hint?: string;
}

export function EmptyState({ message, hint }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-start gap-2 py-8">
      <p className="text-base text-text-secondary">{message}</p>
      {hint ? <p className="text-sm text-text-tertiary">{hint}</p> : null}
    </div>
  );
}
