/*
 * LayerHost — Anayasa madde 22.7: "bütün modüller aynı evren içindedir".
 *
 * Her modül katmanı bu kabuğa sarılır. Ortak zemin, ortak kenar boşluğu,
 * ortak kaydırma davranışı burada tanımlanır ki modüller "aynı evren"
 * hissini korusun ve her biri kendi layout'unu yeniden icat etmesin.
 *
 * Kaydırma: yalnız DİKEY, yalnız BURADA. Gövde yatay kaymaz (madde 35.5).
 */

interface LayerHostProps {
  children: React.ReactNode;
  /** Katman başlığı — Zoom Trail zaten gösterir; burası ekran içi başlık (opsiyonel). */
  title?: string;
}

export function LayerHost({ children, title }: LayerHostProps) {
  return (
    <div className="h-full w-full overflow-y-auto overflow-x-hidden bg-surface-1">
      {/* Dock ve Trail için alt/üst nefes payı — token'lardan (madde 29.3) */}
      <div className="mx-auto w-full max-w-[1120px] px-8 pb-16 pt-12">
        {title ? (
          <h1 className="mb-8 text-xl font-semibold text-text-primary">{title}</h1>
        ) : null}
        {children}
      </div>
    </div>
  );
}
