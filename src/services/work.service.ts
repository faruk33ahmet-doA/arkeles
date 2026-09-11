import { ipc } from "./ipc";
import { ipcStrict } from "./ipc";
import type {
  DocumentRef,
  MeetingSummary,
  NoteDetail,
  NoteSummary,
  Task,
  Workspace,
  WorkspaceOverview,
} from "@/lib/generated";

/*
 * İş modülü servisi — Sprint 3.
 *
 * KURUMA ÖZEL FONKSİYON YOKTUR. Her çağrı `workspaceId` alır; WIF ve GEN
 * aynı yoldan geçer (Sprint 3 madde 1: "WIF veya GEN için özel, tekrar
 * kullanılamaz kod yazma").
 */

const EMPTY_OVERVIEW = (workspaceId: string): WorkspaceOverview => ({
  workspaceId,
  activeTasks: 0,
  waitingTasks: 0,
  doneTasks: 0,
  lastActivity: null,
  priority: null,
  recentNotes: [],
  nextMeeting: null,
});

export async function listWorkspaces(): Promise<Workspace[]> {
  return ipc<Workspace[]>("list_workspaces", []);
}

export async function getWorkspaceOverview(
  workspaceId: string,
): Promise<WorkspaceOverview> {
  return ipc<WorkspaceOverview>("workspace_overview", EMPTY_OVERVIEW(workspaceId), {
    workspaceId,
  });
}

export async function getWorkspaceTasks(workspaceId: string): Promise<Task[]> {
  return ipc<Task[]>("workspace_tasks", [], { workspaceId });
}

/** `kind`: "note" | "project" — Sprint 3 madde 7, 10. */
export async function getWorkspaceNotes(
  workspaceId: string,
  kind: "note" | "project",
): Promise<NoteSummary[]> {
  return ipc<NoteSummary[]>("workspace_notes", [], { workspaceId, kind });
}

export async function getWorkspaceMeetings(
  workspaceId: string,
): Promise<MeetingSummary[]> {
  return ipc<MeetingSummary[]>("workspace_meetings", [], { workspaceId });
}

export async function getWorkspaceDocuments(
  workspaceId: string,
): Promise<DocumentRef[]> {
  return ipc<DocumentRef[]>("workspace_documents", [], { workspaceId });
}

export async function getNoteDetail(noteId: string): Promise<NoteDetail> {
  return ipcStrict<NoteDetail>("note_detail", { noteId });
}

/*
 * Belgeyi işletim sistemine açtırır — Sprint 3 madde 8.
 *
 * `ipcStrict`: kullanıcının bilinçli eylemi, sessizce başarısız olamaz.
 * ARKELÉS belgeyi RENDER ETMEZ; dosyayı sahibi olan uygulamaya devreder
 * (madde 7.3: ARKELÉS belge üretmez).
 */
export async function openDocument(documentId: string): Promise<void> {
  return ipcStrict<void>("open_document", { documentId });
}
