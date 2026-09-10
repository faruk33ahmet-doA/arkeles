/*!
Vault katmanı — Anayasa madde 7.2, 8.1.

Obsidian vault'u TEK DOĞRULUK KAYNAĞIDIR (madde 7.2). Bu modül ona
dokunan tek yerdir; başka hiçbir modül dosya sistemine erişmez.

  reader      → tarama + tek not okuma
  frontmatter → YAML ayrıştırma (bozuk YAML notu düşürmez)
  tasks       → `- [ ]` / `- [x]`
  links       → `[[hedef]]`
  id          → ULID kimlik tanıma (madde 16)
  time        → ISO 8601 (madde 14.1: chrono eklemiyoruz)
  writer      → mekanik yazma (Sprint 2)
*/

pub mod frontmatter;
pub mod id;
pub mod links;
pub mod note;
pub mod reader;
pub mod tasks;
pub mod time;
pub mod writer;
