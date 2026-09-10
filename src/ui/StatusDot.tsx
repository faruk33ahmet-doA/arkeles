import { cn } from "@/lib/cn";

/*
 * StatusDot — Anayasa madde 18.3 (Hermes durum göstergesi), 25.2.
 *
 * Bilgi DURUM olarak gösterilir, uyarı olarak değil. Bu yüzden:
 *  - "offline" gri, kırmızı DEĞİL (madde 18.4: kapalı olmak hata değildir)
 *  - animasyon YOK: nefes alan/pulse eden gösterge yasak (madde 23.2)
 */

type Status = "online" | "offline" | "attention";

const colorByStatus: Record<Status, string> = {
  online: "bg-status-online",
  offline: "bg-status-offline",
  attention: "bg-status-attention",
};

export function StatusDot({ status, label }: { status: Status; label: string }) {
  return (
    <span className="inline-flex items-center gap-2">
      <span
        aria-hidden
        className={cn("h-[6px] w-[6px] rounded-full", colorByStatus[status])}
      />
      <span className="text-sm text-text-secondary">{label}</span>
    </span>
  );
}
