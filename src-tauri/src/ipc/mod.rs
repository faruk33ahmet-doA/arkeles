/*!
IPC yüzeyi — Anayasa madde 12, 14.2.

Webview'e açılan TÜM komutlar burada toplanır. Yüzey BİLİNÇLİ OLARAK DARDIR:
her yeni komut, anayasa madde 8 yetki ayrımına göre gerekçelendirilmelidir.

Madde 14.2: Bu komutların dönüş tipleri Sprint 1'den itibaren `ts-rs` ile
TypeScript'e ÜRETİLECEK. Elle yazılmış tip tanımı yasaktır; Sprint 0'da
frontend tarafındaki karşılıkları "GEÇİCİ: ts-rs Sprint 1" ile işaretlidir.
*/

pub mod hermes_cmds;
pub mod mutation_cmds;
pub mod work_cmds;
pub mod index_cmds;
pub mod vault_cmds;
