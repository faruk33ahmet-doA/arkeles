import { ZoomEngine } from "@/navigation/zoom/ZoomEngine";
import { Dock } from "@/navigation/dock/Dock";
import { ZoomTrail } from "@/navigation/trail/ZoomTrail";
import { CommandPalette } from "@/command/CommandPalette";
import { useCommandPaletteHotkey } from "@/command/useCommandPalette";
import { useGlobalKeybindings } from "./keybindings";
import { WindowDragRegion } from "./WindowDragRegion";

/*
 * AppShell — Anayasa madde 22.7: "bütün modüller aynı evren içindedir".
 *
 * Kabuk HİÇ unmount olmaz. Dock, Trail ve Palet katman geçişlerinden
 * etkilenmez; yalnız içerideki katman değişir. Bu, "yeni pencere hissi
 * oluşmaz" kuralının teknik karşılığıdır.
 *
 * 21.2  Splash / yükleniyor / hoş geldin ekranı YOK — kabuk doğrudan gelir.
 */

export function AppShell() {
  useCommandPaletteHotkey();
  useGlobalKeybindings();

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-surface-0">
      <ZoomEngine />
      {/* Sürükleme şeridi z-drag'de: trail ve dock'un altında (bkz. bileşen). */}
      <WindowDragRegion />
      <ZoomTrail />
      <Dock />
      <CommandPalette />
    </div>
  );
}
