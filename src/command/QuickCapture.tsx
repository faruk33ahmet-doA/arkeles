import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { MutationError, quickCapture } from "@/services/mutation.service";
import { queryKeys } from "@/data/queryKeys";
import { cn } from "@/lib/cn";

/*
 * Hızlı Yakalama — Anayasa madde 10.
 *
 * 10.1  Cmd+K içinden erişilir.
 * 10.2  Vault içindeki gelen kutusu dosyasına HAM BİR SATIR ekler.
 *       Yapı kurmaz, sınıflandırmaz, başlık atmaz, frontmatter yazmaz.
 * 10.3  ARKELÉS metne ANLAM VERMEZ — yalnız kaydeder.
 * 10.4  Ham satıra anlam vermek HERMES'in işidir.
 *
 * Bu yüzden burada: etiket seçici yok, kategori yok, tarih seçici yok,
 * "görev olarak ekle" seçeneği yok. Tek bir metin alanı ve tek bir eylem.
 * Her ek alan madde 10.2'nin ihlali olurdu.
 */

interface QuickCaptureProps {
  /** Gelen kutusu yolu — kullanıcı nereye yazdığını bilmeli. */
  inboxPath: string;
  onDone: () => void;
}

export function QuickCapture({ inboxPath, onDone }: QuickCaptureProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: (value: string) => quickCapture(value),
    onSuccess: () => {
      setText("");
      setError(null);
      // Gelen kutusu bir nottur: index'teki hali tazelenmeli.
      queryClient.invalidateQueries({ queryKey: queryKeys.today.view });
      onDone();
    },
    onError: (err) => {
      // Madde 25.4: kullanıcının bilinçli eylemi sessizce başarısız olamaz.
      if (err instanceof MutationError && err.code === "inbox_missing") {
        setError(`${inboxPath} bulunamadı.`);
        return;
      }
      setError(err instanceof Error ? err.message : "Yazılamadı.");
    },
  });

  const submit = () => {
    const value = text.trim();
    if (!value || mutation.isPending) return;
    mutation.mutate(value);
  };

  return (
    <div className="flex flex-col gap-3 p-4">
      <textarea
        autoFocus
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={(event) => {
          // Enter kaydeder, Shift+Enter satır ekler (metin tek satıra
          // indirilecek ama kullanıcının yazma akışını kesmeyiz).
          if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            submit();
          }
        }}
        rows={3}
        placeholder="Aklına geleni yaz…"
        className={cn(
          "w-full resize-none select-text rounded-sm bg-transparent text-base",
          "text-text-primary outline-none placeholder:text-text-tertiary",
        )}
      />

      <div className="flex items-center justify-between gap-4">
        {/* Nereye yazıldığı görünür: kullanıcı ne olduğunu bilmeli. */}
        <span className="min-w-0 truncate text-xs text-text-tertiary">
          {error ?? `${inboxPath} · ham satır olarak eklenir`}
        </span>

        <button
          type="button"
          onClick={submit}
          disabled={!text.trim() || mutation.isPending}
          className={cn(
            "shrink-0 rounded-sm border border-border-default px-3 py-1 text-sm font-medium",
            "text-text-primary transition-colors duration-fast ease-out",
            "outline-none hover:bg-surface-3 focus-visible:ring-1 focus-visible:ring-border-strong",
            "disabled:cursor-default disabled:text-text-tertiary",
          )}
        >
          {mutation.isPending ? "Ekleniyor…" : "Ekle"}
        </button>
      </div>
    </div>
  );
}
