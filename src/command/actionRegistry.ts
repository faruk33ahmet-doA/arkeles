import type { LayerId } from "@/navigation/layers/types";
import type { WorkPanel } from "@/state/workStore";
import type { Workspace } from "@/lib/generated";
import { listLayers } from "@/navigation/layers/layerRegistry";

/*
 * Aksiyon Kaydı — Anayasa madde 27.3:
 * "Bütün aksiyonlar merkezi bir aksiyon kaydından tanımlanır.
 *  Komut paleti bu kaydı okur. Kayıtta olmayan aksiyon yoktur."
 *
 * Bu kayıt Sprint 0'da kurulur çünkü sonradan kurulan komut paletleri
 * hep eksik kalır (bkz. teknik değerlendirme, A7 gerekçesi).
 *
 * Aksiyon türleri anayasadaki yetki ayrımını (madde 8) yansıtır:
 *   "navigate"  → katman değiştirir, yazma yok
 *   "mechanic"  → mekanik mutasyon, ARKELÉS yazar (madde 8.1) — Sprint 2
 *   "semantic"  → semantik mutasyon, Hermes yapar (madde 8.2) — Sprint 4
 *   "system"    → uygulama yapılandırması (madde 17.2)
 *
 * Anayasa madde 18.2: Hermes'in bildirmediği yetenek arayüzde HİÇ görünmez.
 * Bu yüzden "semantic" aksiyonlar bir `capability` anahtarı taşır ve
 * palet, Hermes'in yetenek listesinde olmayanları filtreler.
 */

export type ActionKind = "navigate" | "mechanic" | "semantic" | "system";

export interface Action {
  id: string;
  /** Palette görünen ad. */
  title: string;
  /** Gruplama başlığı. */
  group: string;
  kind: ActionKind;
  /** Arama eşleşmesini genişletir (Türkçe/İngilizce eş anlam). */
  keywords?: string[];
  /** Yalnız "semantic" için: Hermes yetenek anahtarı (madde 18.2). */
  capability?: string;
  /** Aksiyonun kendisi. Sprint 0'da navigate dışındakiler tanımsız. */
  run?: (ctx: ActionContext) => void | Promise<void>;
}

export interface ActionContext {
  enterLayer: (layer: LayerId) => void;
  closePalette: () => void;
  /** Hızlı Yakalama panelini açar — madde 10.1. */
  openQuickCapture: () => void;
  /** İş modülünde bir çalışma alanı ve panel açar — Sprint 3 madde 13. */
  openWorkspace: (workspaceId: string, panel?: WorkPanel) => void;
}

/** Katmanlardan otomatik türeyen navigasyon aksiyonları (madde 27.4). */
function navigationActions(): Action[] {
  return listLayers().map((layer) => ({
    id: `navigate:${layer.id}`,
    title: layer.title,
    group: "Git",
    kind: "navigate" as const,
    keywords: [layer.id],
    run: (ctx) => {
      ctx.enterLayer(layer.id);
      ctx.closePalette();
    },
  }));
}

/*
 * Çalışma alanı aksiyonları — Sprint 3 madde 13.
 *
 * "Hardcode edilmiş paralel navigasyon sistemi oluşturma."
 *
 * Bu yüzden aksiyonlar çekirdekten gelen kurum LİSTESİNDEN türer; WIF ve
 * GEN için elle yazılmış giriş YOKTUR. Yeni bir kurum aktifleşince
 * aksiyonları kendiliğinden belirir.
 *
 * Yalnız AKTİF kurumlar için "görevlerini aç" aksiyonu üretilir — pasif
 * kurumda o panel boş olurdu ve madde 18.2'yi ihlal ederdi.
 */
function workspaceActions(workspaces: Workspace[]): Action[] {
  const actions: Action[] = [];

  for (const ws of workspaces) {
    actions.push({
      id: `work:${ws.id}`,
      title: `${ws.label}'e git`,
      group: "İş",
      kind: "navigate",
      keywords: [ws.id, ws.label, "iş", "kurum", "çalışma alanı"],
      run: (ctx) => {
        ctx.openWorkspace(ws.id);
        ctx.closePalette();
      },
    });

    if (ws.active) {
      actions.push({
        id: `work:${ws.id}:tasks`,
        title: `${ws.label} görevlerini aç`,
        group: "İş",
        kind: "navigate",
        keywords: [ws.id, ws.label, "görev", "task"],
        run: (ctx) => {
          ctx.openWorkspace(ws.id, "tasks");
          ctx.closePalette();
        },
      });
    }
  }

  return actions;
}

/*
 * Hızlı Yakalama — Anayasa madde 10. MEKANİK bir işlemdir (10.3), bu yüzden
 * Hermes yeteneği GEREKTİRMEZ ve Hermes kapalıyken de çalışır (madde 18.3).
 *
 * Ama gelen kutusu DOSYASI yoksa görünmez: ARKELÉS o dosyayı oluşturamaz
 * (10.2) ve madde 18.2 "tıklandığında hata veren buton yoktur" der.
 */
function quickCaptureAction(): Action {
  return {
    id: "capture:quick",
    title: "Hızlı Yakalama",
    group: "Yakala",
    kind: "mechanic",
    keywords: ["not", "ekle", "yakala", "capture", "inbox", "gelen kutusu"],
    run: (ctx) => ctx.openQuickCapture(),
  };
}

/**
 * Palette gösterilecek aksiyonlar.
 *
 * `inboxAvailable`: madde 10.2 + 18.2 — gelen kutusu dosyası yoksa
 * Hızlı Yakalama listelenmez.
 */
export function getActions(options: {
  inboxAvailable: boolean;
  workspaces?: Workspace[];
}): Action[] {
  const actions = [...navigationActions()];

  if (options.inboxAvailable) {
    actions.push(quickCaptureAction());
  }
  // Sprint 3 madde 13: kurum aksiyonları listeden TÜRETİLİR, yazılmaz.
  actions.push(...workspaceActions(options.workspaces ?? []));

  return actions;
}
