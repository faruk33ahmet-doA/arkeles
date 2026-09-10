import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { selectVault } from "@/services/vault.service";
import { VAULT_DEPENDENT_KEYS } from "@/data/queryKeys";
import { isTauri } from "@/services/ipc";
import { cn } from "@/lib/cn";

/*
 * Vault seçim ekranı — Anayasa madde 17.3, 21.
 *
 * 17.3  "Vault yolu hiçbir yerde sabit kodlanmaz. İlk açılışta sorulur."
 * 21.1  Kullanıcı "yeni bir program açmış gibi" hissetmez.
 * 21.2  Splash / hoş geldin / tanıtım turu YOK.
 *
 * Bu yüzden burası bir KARŞILAMA EKRANI DEĞİL. Tek bir soru, tek bir
 * eylem. Logo yok, slogan yok, adım göstergesi yok, "başlayalım" yok.
 * Kullanıcı vault'unu seçer ve bir daha bu ekranı görmez.
 */

export function VaultPicker() {
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: async () => {
      // Native klasör seçici. Madde 19: webview'e dosya sistemi yetkisi
      // verilmez; seçim diyalogu OS'a ait, dönen tek şey bir yol dizgesi.
      const picked = await open({
        directory: true,
        multiple: false,
        title: "Obsidian vault klasörünü seç",
      });
      if (typeof picked !== "string") return null;
      return selectVault(picked);
    },
    onSuccess: (result) => {
      if (!result) return; // kullanıcı vazgeçti — hata değil
      setError(null);
      for (const key of VAULT_DEPENDENT_KEYS) {
        queryClient.invalidateQueries({ queryKey: key });
      }
    },
    onError: (err) => {
      // Madde 25.4: kullanıcının bilinçli eylemi sessizce başarısız olamaz.
      setError(err instanceof Error ? err.message : "Klasör okunamadı.");
    },
  });

  return (
    <div className="flex h-full w-full items-center justify-center bg-surface-1 px-8">
      <div className="w-full max-w-[420px]">
        <h1 className="mb-2 text-lg font-semibold text-text-primary">
          Vault klasörünü seç
        </h1>
        <p className="mb-6 text-base text-text-secondary">
          ARKELÉS notlarını buradan okur. Hiçbir şey kopyalanmaz, hiçbir şey
          taşınmaz.
        </p>

        <button
          type="button"
          onClick={() => mutation.mutate()}
          disabled={mutation.isPending || !isTauri()}
          className={cn(
            "w-full rounded-md border border-border-strong px-4 py-3 text-base font-medium",
            "text-text-primary transition-colors duration-fast ease-out",
            "outline-none hover:bg-surface-3 focus-visible:ring-1 focus-visible:ring-border-strong",
            "disabled:cursor-default disabled:text-text-tertiary",
          )}
        >
          {mutation.isPending ? "Taranıyor…" : "Klasör seç"}
        </button>

        {/* Madde 25.2: hata durum olarak gösterilir, modal olarak değil. */}
        {error ? (
          <p className="mt-4 text-sm text-status-attention">{error}</p>
        ) : null}

        {!isTauri() ? (
          <p className="mt-4 text-sm text-text-tertiary">
            Klasör seçimi yalnız masaüstü uygulamasında çalışır.
          </p>
        ) : null}
      </div>
    </div>
  );
}
