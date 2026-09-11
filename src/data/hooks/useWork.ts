import { useQuery } from "@tanstack/react-query";
import {
  getNoteDetail,
  getWorkspaceDocuments,
  getWorkspaceMeetings,
  getWorkspaceNotes,
  getWorkspaceOverview,
  getWorkspaceTasks,
  listWorkspaces,
} from "@/services/work.service";
import { queryKeys } from "@/data/queryKeys";

/*
 * İş modülü veri kancaları — Sprint 3.
 *
 * Hepsi tek şablon: `workspaceId` parametreli, `enabled` ile boş kimlikte
 * çekirdeğe gitmeyen, retry'sız sorgular. Kuruma özel kanca YOKTUR.
 *
 * Tazeleme dosya izleyiciden gelir (`vault:changed`), yoklamayla değil
 * (madde 35.1).
 */

export function useWorkspaces() {
  return useQuery({
    queryKey: queryKeys.work.workspaces,
    queryFn: listWorkspaces,
    retry: false,
  });
}

export function useWorkspaceOverview(workspaceId: string | null) {
  return useQuery({
    queryKey: queryKeys.work.overview(workspaceId ?? ""),
    queryFn: () => getWorkspaceOverview(workspaceId!),
    enabled: Boolean(workspaceId),
    retry: false,
  });
}

export function useWorkspaceTasks(workspaceId: string | null) {
  return useQuery({
    queryKey: queryKeys.work.tasks(workspaceId ?? ""),
    queryFn: () => getWorkspaceTasks(workspaceId!),
    enabled: Boolean(workspaceId),
    retry: false,
  });
}

export function useWorkspaceNotes(
  workspaceId: string | null,
  kind: "note" | "project",
) {
  return useQuery({
    queryKey: queryKeys.work.notes(workspaceId ?? "", kind),
    queryFn: () => getWorkspaceNotes(workspaceId!, kind),
    enabled: Boolean(workspaceId),
    retry: false,
  });
}

export function useWorkspaceMeetings(workspaceId: string | null) {
  return useQuery({
    queryKey: queryKeys.work.meetings(workspaceId ?? ""),
    queryFn: () => getWorkspaceMeetings(workspaceId!),
    enabled: Boolean(workspaceId),
    retry: false,
  });
}

export function useWorkspaceDocuments(workspaceId: string | null) {
  return useQuery({
    queryKey: queryKeys.work.documents(workspaceId ?? ""),
    queryFn: () => getWorkspaceDocuments(workspaceId!),
    enabled: Boolean(workspaceId),
    retry: false,
  });
}

export function useNoteDetail(noteId: string | null) {
  return useQuery({
    queryKey: queryKeys.work.noteDetail(noteId ?? ""),
    queryFn: () => getNoteDetail(noteId!),
    enabled: Boolean(noteId),
    retry: false,
  });
}
