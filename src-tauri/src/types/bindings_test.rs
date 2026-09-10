/*!
ts-rs tip üretimi — Anayasa madde 14.2.

"Elle yazılmış IPC tip tanımı yasaktır. Sözleşme tek yerde yaşar,
 sürüklenme (drift) olmaz."

`ts-rs`, tipleri bir TEST çalıştırması sırasında dışa aktarır. Bu yüzden
üretim komutu:

    cargo test export_bindings

Çıktı: src/lib/generated/*.ts

Bu bir test dosyasıdır ama hiçbir şeyi doğrulamaz — `#[ts(export)]`
niteliğinin çalışması için bir test koşumu gerekir, o kadar.
*/
