import { EmptyState } from "@/ui/EmptyState";
import { VirtualList } from "@/ui/VirtualList";
import { useWorkStore } from "@/state/workStore";
import { cn } from "@/lib/cn";
import type { NoteSummary } from "@/lib/generated";

/*
 * Not listesi — Projeler ve Notlar panelleri AYNI bileşeni kullanır.
 *
 * Sprint 3 madde 14: "önce gerçek tekrar oluşsun, sonra abstraction yap."
 * Tekrar iki panelde oluştu; soyutlama burada ve BİR SEVİYE derin değil.
 */

interface NoteListProps {
  notes: NoteSummary[];
  emptyMessage: string;
  emptyHint?: string;
  loading?: boolean;
}

export function NoteList({ notes, emptyMessage, emptyHint, loading }: NoteListProps) {
  const openNote = useWorkStore((s) => s.openNote);

  if (notes.length === 0) {
    return loading ? null : <EmptyState message={emptyMessage} hint={emptyHint} />;
  }

  return (
    <VirtualList
      items={notes}
      keyOf={(note) => note.id}
      renderItem={(note) => (
        <button
          type="button"
          onClick={() => openNote(note.id)}
          className={cn(
            "flex w-full items-center justify-between gap-4 py-2 text-left",
            "outline-none focus-visible:ring-1 focus-visible:ring-border-strong",
          )}
        >
          <span className="min-w-0 flex-1 truncate text-base text-text-primary">
            {note.title}
          </span>
          <span className="shrink-0 text-xs tabular-nums text-text-tertiary">
            {note.modifiedAt.slice(0, 10)}
          </span>
          {/* Madde 16.3: yönetilmeyen not salt okunur. */}
          {!note.managed ? (
            <span className="shrink-0 text-xs text-text-tertiary">yönetilmiyor</span>
          ) : null}
        </button>
      )}
    />
  );
}
