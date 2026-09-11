import { Card } from "@/ui/Card";
import { EmptyState } from "@/ui/EmptyState";
import { VirtualList } from "@/ui/VirtualList";
import { useWorkspaceMeetings } from "@/data/hooks/useWork";
import { useWorkStore } from "@/state/workStore";
import { cn } from "@/lib/cn";

/*
 * Toplantılar — Sprint 3 madde 9.
 *
 * "Google Calendar entegrasyonu YOK. Yeni toplantı oluşturma YOK.
 *  Sadece mevcut Obsidian verisi."
 *
 * Bir not ancak `arkeles_type: meeting` VE `date` alanı taşıyorsa burada
 * görünür. Tarihsiz bir "toplantı" takvimde yeri olmayan bir nottur;
 * tarih uydurmak veri üretmek olurdu (madde 7.3).
 */

export function MeetingsPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceMeetings(workspaceId);
  const openNote = useWorkStore((s) => s.openNote);

  const meetings = data ?? [];

  return (
    <Card title="Toplantılar">
      {meetings.length > 0 ? (
        <VirtualList
          items={meetings}
          keyOf={(meeting) => meeting.id}
          renderItem={(meeting) => (
            <button
              type="button"
              onClick={() => openNote(meeting.id)}
              className={cn(
                "flex w-full items-center justify-between gap-4 py-2 text-left",
                "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
              )}
            >
              <span className="min-w-0 flex-1 truncate text-base text-text-primary">
                {meeting.title}
              </span>
              <span className="shrink-0 text-xs tabular-nums text-text-tertiary">
                {meeting.date}
              </span>
            </button>
          )}
        />
      ) : isLoading ? null : (
        <EmptyState
          message="Kayıtlı toplantı yok."
          hint="arkeles_type: meeting ve date alanı taşıyan notlar burada görünür."
        />
      )}
    </Card>
  );
}
