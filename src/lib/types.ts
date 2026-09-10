/*
 * Paylaşılan tip tanımları.
 *
 * Anayasa madde 14.2: IPC tipleri ELLE YAZILMAZ. Sprint 1'den itibaren
 * Rust çekirdeğinden `ts-rs` ile üretilip buraya (src/lib/generated/) gelir.
 * Sprint 0'da IPC yüzeyi yalnız mock olduğundan tipler geçici olarak
 * ilgili servis dosyalarında elle tanımlıdır ve "// GEÇİCİ: ts-rs Sprint 1"
 * yorumuyla işaretlidir.
 */

/** Bir değerin yüklenme durumu — UI'da spinner yerine hiyerarşi için (madde 23.2, 28). */
export type Freshness = "fresh" | "stale" | "unknown";
