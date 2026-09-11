import { useState } from "react";
import { useCancelJob, useRetryJob } from "@/data/hooks/useHermes";
import { cn } from "@/lib/cn";
import type { Job } from "@/lib/generated";

/*
 * Bir iş satırı — Sprint 4 madde 3, 7, 15, 16.
 *
 * 7   SAHTE PROGRESS YOK: Hermes yüzde vermiyorsa yalnız durum adı görünür.
 * 15  Hata sessizce kaybolmaz ama KIRMIZI ALARM da yapılmaz; sönük amber.
 * 16  İptal yalnız queued/running işte görünür.
 *
 * Sonuç referansı satırı YOK (Sprint 5): gerçek Hermes yapısal çıktı
 * bildirmiyor. Hermes'in yazdığı notlar vault'a düşer ve iş bitince vault
 * tazelenir — ARKELÉS sonuç UYDURMAZ (madde 8).
 */

const STATUS_LABEL: Record<string, string> = {
  queued: "Bekliyor",
  running: "Çalışıyor",
  completed: "Tamamlandı",
  failed: "Başarısız",
  cancelled: "İptal edildi",
};

/*
 * Durum rengi — anayasa madde 25.2, 32.
 * KIRMIZI HİÇ KULLANILMAZ. Başarısızlık sönük amber (veri kaybı riski için
 * ayrılmış tek vurgu), diğerleri nötr.
 */
const STATUS_TONE: Record<string, string> = {
  queued: "text-text-tertiary",
  running: "text-text-primary",
  completed: "text-text-secondary",
  failed: "text-status-attention",
  cancelled: "text-text-tertiary",
};

export function JobRow({ job }: { job: Job }) {
  const retry = useRetryJob();
  const cancel = useCancelJob();
  const [detail, setDetail] = useState(false);

  const isActive = job.status === "queued" || job.status === "running";
  const label = STATUS_LABEL[job.status] ?? job.status;

  return (
    <div className="flex flex-col gap-2 py-3">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <p className="truncate text-base text-text-primary">{job.summary}</p>
          <p className="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-tertiary">
            <span>{job.actionLabel}</span>
            {job.workspace ? <span>· {job.workspace}</span> : null}
            {/* Sprint 4 madde 17: kaynak. Hermes bildirmiyorsa "arkeles" kalır. */}
            {job.source !== "arkeles" ? <span>· {job.source}</span> : null}
          </p>
        </div>

        <div className="flex shrink-0 items-center gap-3">
          {/* Madde 7: progress varsa göster, YOKSA UYDURMA. */}
          {job.progress !== null ? (
            <span className="text-xs tabular-nums text-text-secondary">
              %{job.progress}
            </span>
          ) : null}
          <span className={cn("text-xs", STATUS_TONE[job.status] ?? "text-text-secondary")}>
            {label}
          </span>
        </div>
      </div>

      {/* Madde 15: hata görünür ama akışı kesmez. */}
      {job.status === "failed" && detail && job.errorMessage ? (
        <p className="select-text rounded-sm bg-surface-3 p-2 text-xs text-text-secondary">
          {job.errorMessage}
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2">
        {job.status === "failed" ? (
          <>
            <button
              type="button"
              onClick={() => retry.mutate(job.id)}
              disabled={retry.isPending}
              className={actionButton}
            >
              Yeniden dene
            </button>
            {job.errorMessage ? (
              <button
                type="button"
                onClick={() => setDetail((value) => !value)}
                className={actionButton}
              >
                {detail ? "Detayı gizle" : "Detay"}
              </button>
            ) : null}
          </>
        ) : null}

        {isActive ? (
          <button
            type="button"
            onClick={() => cancel.mutate(job.id)}
            disabled={cancel.isPending}
            className={actionButton}
          >
            İptal
          </button>
        ) : null}
      </div>
    </div>
  );
}

const actionButton = cn(
  "rounded-sm border border-border-default px-2 py-1 text-xs font-medium",
  "text-text-secondary transition-colors duration-fast ease-out",
  "outline-none hover:bg-surface-3 hover:text-text-primary",
  "focus-visible:ring-1 focus-visible:ring-border-strong",
  "disabled:cursor-default disabled:text-text-tertiary",
);
