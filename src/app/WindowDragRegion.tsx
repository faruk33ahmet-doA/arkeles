/*
 * Pencere sürükleme bölgesi — Anayasa madde 35.5: "Native his korunur."
 *
 * NEDEN GEREKLİ: tauri.conf.json'da `titleBarStyle: "Overlay"` +
 * `hiddenTitle: true` kullanıyoruz — çünkü madde 21.1 "kullanıcı yeni bir
 * program açmış gibi hissetmez, kendi çalışma alanına girmiş gibi hisseder"
 * diyor ve klasik başlık çubuğu bunu bozar.
 *
 * Ama sistem başlık çubuğu kalkınca pencereyi TAŞIMANIN yolu da kalkar.
 * Taşınamayan bir masaüstü penceresi native değildir. Bu şerit o boşluğu
 * doldurur: görünmez, içeriği engellemez, yalnız sürükleme sağlar.
 *
 * Yükseklik, macOS trafik ışıklarının bulunduğu bandı kapsar.
 * z katmanı: layer (10) ve panel (20) ÜSTÜNDE, trail (30) / dock (40) /
 * command (50) ALTINDA. Böylece breadcrumb ve dock tıklanabilir kalır;
 * DOM sırasına bağımlı değildir.
 */

export function WindowDragRegion() {
  return (
    <div
      data-tauri-drag-region
      aria-hidden
      className="fixed inset-x-0 top-0 z-drag h-8"
    />
  );
}
