import { useState } from "react";
import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { VirtualList } from "@/ui/VirtualList";
import { useWorkspaceDocuments } from "@/data/hooks/useWork";
import { openDocument } from "@/services/work.service";
import { cn } from "@/lib/cn";

/*
 * Belgeler — Sprint 3 madde 8.
 *
 * ARKELÉS belgeyi AÇMAZ, RENDER ETMEZ, KOPYALAMAZ (madde 7.3). Dosyayı
 * sahibi olan uygulamaya devreder. Bu bilinçli: bir PDF görüntüleyici
 * yazmak ARKELÉS'i "gösteren" olmaktan çıkarıp "yapan" yapardı.
 *
 * Madde 22.7 ("yeni pencere hissi oluşmaz") burada ihlal edilmiyor çünkü
 * belge ARKELÉS'in bir yüzeyi değil, işletim sisteminin bir dosyası —
 * kullanıcı zaten dışarı çıktığını bilir ve bekler.
 *
 * Zoom KULLANILMAZ (Sprint 3 madde 8): bu bir alt yüzeydir.
 */

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function DocumentsPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceDocuments(workspaceId);
  const [error, setError] = useState<string | null>(null);

  const documents = data ?? [];

  const open = (documentId: string) => {
    setError(null);
    // Madde 25.4: kullanıcının bilinçli eylemi sessizce başarısız olamaz.
    openDocument(documentId).catch(() => setError("Dosya açılamadı."));
  };

  return (
    <Card
      title="Belgeler"
      action={
        error ? <span className="text-xs text-status-attention">{error}</span> : null
      }
    >
      {documents.length > 0 ? (
        <VirtualList
          items={documents}
          keyOf={(doc) => doc.id}
          renderItem={(doc) => (
            <button
              type="button"
              onClick={() => open(doc.id)}
              className={cn(
                "flex w-full items-center justify-between gap-4 py-2 text-left",
                "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
              )}
            >
              <span className="min-w-0 flex-1 truncate text-base text-text-primary">
                {doc.fileName}
              </span>
              <span className="shrink-0 text-xs uppercase text-text-tertiary">
                {doc.extension}
              </span>
              <span className="shrink-0 text-xs tabular-nums text-text-tertiary">
                {formatSize(doc.sizeBytes)}
              </span>
            </button>
          )}
        />
      ) : isLoading ? null : (
        <EmptyState
          message="Bağlı belge yok."
          hint="Bir notta [[dosya.pdf]] şeklinde bağlantı kurulduğunda burada görünür."
        />
      )}
    </Card>
  );
}
