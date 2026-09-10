import { useQueryClient } from "@tanstack/react-query";
import { Dialog, DialogContent, DialogOverlay, DialogPortal, DialogTitle } from "@/ui/primitives/dialog";
import { useConflictStore } from "@/state/conflictStore";
import { VAULT_DEPENDENT_KEYS } from "@/data/queryKeys";
import { cn } from "@/lib/cn";

/*
 * Çakışma arayüzü — Anayasa madde 20.3, 25.2, 25.4.
 *
 * 25.4  Bu, akışı kesmeye YETKİLİ tek durumdur — veri kaybı riski var.
 * 25.2  Ama yine de SESSİZ: kırmızı alarm yok, ses yok, toast yığını yok.
 * 23.2  Animasyon yok — nefes alan/pulse eden öğe yasak.
 * 30.1  Cam efekti YOK: dock ve Cmd+K zaten iki cam yüzeyi kullanıyor
 *       (madde 30.3 en fazla 2). Bu panel düz yüzeydir.
 *
 * Renk: `status-attention` (sönük amber) — anayasada veri kaybı riski için
 * ayrılmış tek renk. Kırmızı bu uygulamada HİÇ kullanılmıyor.
 *
 * Varsayılan davranış: HİÇBİR VERİ ÜZERİNE YAZILMAZ. "Üzerine yaz" diye
 * bir seçenek YOKTUR — madde 20.3 bunu yasaklıyor.
 */

export function ConflictPanel() {
  const conflict = useConflictStore((s) => s.conflict);
  const dismiss = useConflictStore((s) => s.dismiss);
  const queryClient = useQueryClient();

  if (!conflict) return null;

  const showCurrent = () => {
    // "Güncel halini göster": index'ten tazele, kullanıcı gerçeği görsün.
    for (const key of VAULT_DEPENDENT_KEYS) {
      queryClient.invalidateQueries({ queryKey: key });
    }
    dismiss();
  };

  const retry = () => {
    conflict.retry?.();
    dismiss();
  };

  return (
    <Dialog open onOpenChange={(next) => !next && dismiss()}>
      <DialogPortal>
        <DialogOverlay className="fixed inset-0 z-conflict bg-black/40" />
        <DialogContent
          className={cn(
            "fixed left-1/2 top-1/2 z-conflict w-full max-w-[440px]",
            "-translate-x-1/2 -translate-y-1/2",
            "rounded-lg border border-border-default bg-surface-2 p-6 shadow-3",
            "outline-none",
          )}
        >
          <DialogTitle className="mb-2 text-base font-medium text-text-primary">
            Bu içerik başka bir kaynak tarafından değiştirildi.
          </DialogTitle>

          <p className="mb-1 text-sm text-text-secondary">{conflict.action}</p>
          <p className="mb-6 text-sm text-text-tertiary">
            Hiçbir şey üzerine yazılmadı.
          </p>

          <div className="flex flex-wrap gap-2">
            <button type="button" onClick={showCurrent} className={primaryButton}>
              Güncel halini göster
            </button>
            {conflict.retry ? (
              <button type="button" onClick={retry} className={secondaryButton}>
                Yeniden dene
              </button>
            ) : null}
            <button type="button" onClick={dismiss} className={secondaryButton}>
              Vazgeç
            </button>
          </div>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}

const baseButton = cn(
  "rounded-sm px-3 py-2 text-sm font-medium transition-colors duration-fast ease-out",
  "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
);

const primaryButton = cn(
  baseButton,
  "border border-status-attention/40 bg-accent-muted text-text-primary hover:bg-surface-3",
);

const secondaryButton = cn(
  baseButton,
  "border border-border-default text-text-secondary hover:text-text-primary hover:bg-surface-3",
);
