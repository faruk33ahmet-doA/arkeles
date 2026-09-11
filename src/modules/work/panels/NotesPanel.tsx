import { Card } from "@/ui/Card";
import { NoteList } from "../components/NoteList";
import { useWorkspaceNotes } from "@/data/hooks/useWork";

/** Notlar — Sprint 3 madde 10. */
export function NotesPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceNotes(workspaceId, "note");

  return (
    <Card
      title="Notlar"
      action={
        data && data.length > 0 ? (
          <span className="text-xs tabular-nums text-text-tertiary">{data.length}</span>
        ) : null
      }
    >
      <NoteList
        notes={data ?? []}
        loading={isLoading}
        emptyMessage="Bu çalışma alanına bağlı not yok."
        emptyHint="Notun frontmatter'ında workspace alanı olmalı."
      />
    </Card>
  );
}
