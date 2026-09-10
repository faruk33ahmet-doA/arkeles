/*!
Vault katmanı — Anayasa madde 7.2, 8.1.

Obsidian vault'u TEK DOĞRULUK KAYNAĞIDIR (madde 7.2). Bu modül ona
dokunan tek yerdir; başka hiçbir modül dosya sistemine erişmez.

Alt modüller:
  reader → okuma (Sprint 0: erişilebilirlik + artefakt filtresi)
  writer → mekanik yazma (Sprint 2)
  id     → ULID kimlik tanıma (madde 16)
*/

pub mod id;
pub mod reader;
pub mod writer;
