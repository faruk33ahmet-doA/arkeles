import { Card } from "@/ui/Card";
import { NoteList } from "../components/NoteList";
import { useWorkspaceNotes } from "@/data/hooks/useWork";

/*
 * Projeler — Sprint 3 madde 7.
 *
 * "Yalnızca vault'ta proje olarak tanımlanmış gerçek veri varsa gösterilsin.
 *  Yeni proje oluşturma YOK."
 *
 * Bir not ancak frontmatter'ında `arkeles_type: project` varsa projedir.
 * Başlığında "proje" geçmesi onu proje YAPMAZ — bu bir çıkarsama olurdu
 * ve madde 8.2 uyarınca yargı Hermes'in işidir.
 */

export function ProjectsPanel({ workspaceId }: { workspaceId: string }) {
  const { data, isLoading } = useWorkspaceNotes(workspaceId, "project");

  return (
    <Card title="Projeler">
      <NoteList
        notes={data ?? []}
        loading={isLoading}
        emptyMessage="Tanımlı proje yok."
        emptyHint="Bir notun frontmatter'ına arkeles_type: project eklendiğinde burada görünür."
      />
    </Card>
  );
}
