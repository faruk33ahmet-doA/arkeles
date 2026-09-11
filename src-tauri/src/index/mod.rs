/*!
Index katmanı — Anayasa madde 9, 15.

Bu modül TÜRETİLMİŞ VERİ tutar. Kaynak Obsidian'dır (madde 7.2).
Vault → index akışı TEK YÖNLÜDÜR (madde 15.4).

Silme testi (madde 9.3): index dosyasını silince hiçbir BİLGİ kaybolmaz —
yalnız hız kaybolur, o da vault'tan yeniden taranarak geri gelir.
*/

mod builder;
mod db;
mod query;
pub(crate) mod schema;
mod work;

#[cfg(test)]
mod query_test;
#[cfg(test)]
mod work_test;
#[cfg(test)]
mod stress_test;

pub use builder::ScanReport;
pub use query::{TaskTarget, WriteTarget};
pub use db::IndexHandle;
