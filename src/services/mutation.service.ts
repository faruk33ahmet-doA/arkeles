import { ipcStrict } from "./ipc";
import type { InboxStatus, MutationResult } from "@/lib/generated";
import { ipc } from "./ipc";

/*
 * Mekanik mutasyon servisi — Anayasa madde 8.1, 20.
 *
 * Hepsi `ipcStrict`: bunlar kullanıcının BİLİNÇLİ eylemleri, sessizce
 * başarısız olamazlar (madde 25.4). Çekirdek hata döndürürse burada
 * yapılandırılmış bir hataya çevrilir ki arayüz doğru dallanabilsin.
 *
 * SEMANTİK MUTASYON BURAYA EKLENEMEZ (madde 8.2).
 */

/** Çekirdekten gelen yapılandırılmış hata. `code` kararlıdır, metin değil. */
export interface CoreErrorShape {
  code: string;
  message: string;
}

export class MutationError extends Error {
  readonly code: string;

  constructor(shape: CoreErrorShape) {
    super(shape.message);
    this.name = "MutationError";
    this.code = shape.code;
  }

  /** Madde 20.3: çakışma özel davranış ister (çakışma arayüzü). */
  get isConflict(): boolean {
    return this.code === "conflict";
  }
}

function isCoreErrorShape(value: unknown): value is CoreErrorShape {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    typeof (value as { code: unknown }).code === "string"
  );
}

/** Çekirdek hatasını `MutationError`'a çevirir. Bilinmeyen hata yutulmaz. */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await ipcStrict<T>(command, args);
  } catch (raw) {
    if (isCoreErrorShape(raw)) {
      throw new MutationError(raw);
    }
    throw raw;
  }
}

/** Madde 8.3: görev durumu değişimi mekanik mutasyondur. */
export async function setTaskStatus(taskId: string, done: boolean): Promise<MutationResult> {
  return call<MutationResult>("set_task_status", { taskId, done });
}

/** Madde 8.1: frontmatter alanı. `null` → anahtarı kaldır. */
export async function setFrontmatterField(
  noteId: string,
  key: string,
  value: string | number | boolean | string[] | null,
): Promise<MutationResult> {
  return call<MutationResult>("set_frontmatter_field", { noteId, key, value });
}

/** Madde 8.1: etiket ekle/çıkar. */
export async function toggleTag(
  noteId: string,
  tag: string,
  add: boolean,
): Promise<MutationResult> {
  return call<MutationResult>("toggle_tag", { noteId, tag, add });
}

/*
 * Hızlı Yakalama — Anayasa madde 10.
 *
 * ARKELÉS ham satırı ekler, ANLAM VERMEZ (10.3). Gelen kutusu dosyası
 * yoksa OLUŞTURULMAZ (10.2) → `inbox_missing` hatası döner.
 */
export async function quickCapture(text: string): Promise<void> {
  return call<void>("quick_capture", { text });
}

const INBOX_UNKNOWN: InboxStatus = { path: "", exists: false };

/** Madde 18.2: gelen kutusu yoksa Hızlı Yakalama arayüzde GÖRÜNMEZ. */
export async function getInboxStatus(): Promise<InboxStatus> {
  return ipc<InboxStatus>("inbox_status", INBOX_UNKNOWN);
}

/**
 * Madde 8.1: görev sırası — satır taşıma (Sprint 5). Yalnız aynı nottaki
 * iki görev; satırlar ve hash çekirdekte index'ten okunur.
 */
export async function moveTask(taskId: string, targetTaskId: string): Promise<MutationResult> {
  return call<MutationResult>("move_task", { taskId, targetTaskId });
}
