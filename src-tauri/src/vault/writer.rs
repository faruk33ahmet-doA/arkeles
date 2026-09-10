/*!
Vault yazma — Anayasa madde 8.1, 20.

YETKİ SINIRI (madde 8.1 / 8.2):
Bu modül YALNIZCA mekanik mutasyon yapar: checkbox, frontmatter alanı,
sıralama, durum, etiket. Yeni not, yeni belge, PDF, analiz BURADA OLAMAZ —
onlar Hermes'in tekelindedir (madde 8.2). Bu dosyaya "yeni dosya oluştur"
fonksiyonu eklenmesi anayasa ihlalidir.

GÜVENLİK ZORUNLULUKLARI (madde 20):
20.1  Atomic write: geçici dosyaya yaz → fsync → rename.
20.2  Optimistic concurrency: okuma anındaki mtime + içerik hash doğrulanır.
20.3  Uyuşmazlık varsa YAZMAZ. Sessizce üzerine yazmak YASAKTIR.

SPRINT 0 KAPSAMI: yazma YOK. Sprint 0 teslim kriteri "henüz gerçek veri
olmayacak" diyor ve mekanik mutasyonlar Sprint 2 kapsamındadır.

Bu dosya niyet beyanı olarak durur: yazma yolunun tek kapısı burasıdır,
başka hiçbir modül dosyaya yazmaz.
*/

// Sprint 2'de eklenecek yüzey:
//
//   pub struct WriteGuard { expected_hash: String, expected_mtime: SystemTime }
//   pub fn set_task_status(...) -> CoreResult<()>   // madde 8.1
//   pub fn set_frontmatter_field(...) -> CoreResult<()>
//   pub fn append_to_inbox(...) -> CoreResult<()>   // madde 10.2 Hızlı Yakalama
//
// Ortak kural: hepsi WriteGuard alır, atomic_write üzerinden geçer.
