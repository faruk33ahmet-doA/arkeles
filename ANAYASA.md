# ARKELÉS — PROJE ANAYASASI

**Sürüm:** 1.0 (nihai)
**Tarih:** 2026-09-11
**Statü:** Yürürlükte. Bu doküman Prompt 1 ve Prompt 2'yi birleştirir ve **onların yerini alır**.

---

## BÖLÜM 0 — BU DOKÜMAN HAKKINDA

**0.1** Bu, geliştirme sürecinde referans alınacak **tek** anayasadır. Önceki iki prompt
arşivdir; çelişki halinde bu doküman geçerlidir.

**0.2** Maddeler numaralıdır. Kod incelemesinde ret gerekçesi madde numarasıyla verilir.
Örnek: *"Bu geçiş madde 23.2'yi ihlal ediyor."*

**0.3** Bu anayasanın **ürün felsefesi ve kimliği (Bölüm I) değiştirilemez.**
Teknik maddeler (Bölüm III, VI) gerekçeli öneriyle değiştirilebilir; değişiklik
`docs/ADR/` altında karar kaydı olarak yazılır.

**0.4** Bu sürümde çözülen çelişkiler **Ek A**'da listelidir. Çözüm gerekçeleri oradadır.

---

# BÖLÜM I — KİMLİK

## 1. Proje

**ARKELÉS.**

Tek kullanıcılı. Tamamen lokal. Kişisel işletim sistemi.

## 2. Amaç

Hayatın tamamını tek bir uygulamadan yönetebilmek.

Amaç bir üretkenlik uygulaması yapmak **değildir**.
Amaç özellik biriktirmek **değildir**.
Amaç hayatı tek yerden yönetebilmektir.

## 3. Ne Değildir

Notion değildir. Obsidian değildir. Todo uygulaması değildir.
CRM değildir. Jarvis değildir. Klasik bir masaüstü uygulaması değildir.

**Kişisel operasyon merkezidir.**

## 4. Temel Felsefe

> **Hermes çalışır. Obsidian saklar. ARKELÉS gösterir.**

Bu cümle değişmez. Her teknik karar bu cümleye karşı sınanır.

## 5. Ürün Kimliği

ARKELÉS:

- sessizdir
- sakin çalışır
- acele etmez
- bağırmaz
- bildirim yağmuru oluşturmaz
- gösterişli değildir
- kullanıcıyı yormaz
- Apple seviyesinde premium his verir
- modern görünür
- futuristik görünür — ama **bilim kurgu görünmez**
- cyberpunk görünmez
- neon kullanmaz
- gereksiz efekt kullanmaz

## 6. Kimlik Koruma Kuralı

**6.1** Ürün kimliği bundan sonra değişmez.

**6.2** Hiçbir yeni öneri, teknoloji veya özellik bu kimliği bozamaz.
Kimlikle çelişen bir özellik, teknik olarak mümkün olsa bile **eklenmez**.

**6.3** Şüphe halinde varsayılan cevap: **eklemeyiz.**
Bir kişisel işletim sisteminin en büyük düşmanı özellik şişmesidir.

---

# BÖLÜM II — ROLLER VE YETKİ

## 7. Roller

### 7.1 Hermes — beyin

Bütün işleri yapar. AI çalıştırır. Görevleri yerine getirir. Obsidian'a yazar.
Telegram ile haberleşir. Belge, PDF, rapor ve analiz üretir.

**Hermes kullanıcı için görünmezdir.**

### 7.2 Obsidian — gerçek

Tek doğruluk kaynağıdır. Bütün bilgi burada, düz metin olarak yaşar.

### 7.3 ARKELÉS — yüz

Gösterir. Yönetir. Görselleştirir. Hermes ile haberleşir.

**ARKELÉS hiçbir zaman AI olmaz.**
ARKELÉS yeni bilgi üretmez. Belge üretmez. PDF üretmez. Analiz yapmaz.

**ARKELÉS kullanıcı için görünür olan tek şeydir.**

## 8. Yazma Yetkisi *(A3 — kesinleşmiştir)*

Mutasyonlar iki sınıfa ayrılır. Bu ayrım tartışmaya kapalıdır.

### 8.1 Mekanik mutasyon → **ARKELÉS yazar**

Kullanıcının doğrudan ve tek adımlık niyetini dosyaya yansıtan değişiklikler:

- checkbox değişimi
- frontmatter alanı güncelleme
- sıralama
- durum değiştirme
- etiket güncelleme

### 8.2 Semantik mutasyon → **yalnızca Hermes yazar**

Yargı, üretim veya dış dünya teması gerektiren her şey:

- yeni belge
- yeni not
- PDF
- AI üretimi
- Telegram
- rapor
- analiz
- görev oluşturma

### 8.3 Günlük etkileşimlerin sınıflandırması

Kararı netleştirmek için, en sık kullanılan işlemler:

| İşlem | Sınıf | Yetkili |
|---|---|---|
| Görevi tamamla | Mekanik | ARKELÉS |
| Görevin durumunu değiştir | Mekanik | ARKELÉS |
| Görevi yeniden sırala | Mekanik | ARKELÉS |
| Etiket ekle / çıkar | Mekanik | ARKELÉS |
| Notun bir frontmatter alanını değiştir | Mekanik | ARKELÉS |
| **Yeni görev oluştur** | Semantik | Hermes |
| **Yeni not oluştur** | Semantik | Hermes |
| Not gövdesini düzenle | — | **Obsidian** (ARKELÉS metin editörü değildir) |
| Rapor / PDF / analiz | Semantik | Hermes |

**8.4** ARKELÉS bir metin editörü değildir. Not gövdesi Obsidian'da düzenlenir.
ARKELÉS not içeriğine serbest metin yazmaz.

## 9. Türetilmiş Veri *(A2/A4 — kesinleşmiştir)*

**9.1** ARKELÉS **yeni bilgi** üretmez.

**9.2** ARKELÉS **türetilmiş veri** üretebilir: index, önbellek, görünüm durumu.

**9.3 Türetilmiş veri testi:** *Bu veriyi silince bir bilgi kaybolur mu?*
Cevap "hayır" olmak zorundadır. "Evet" ise o veri türetilmiş değildir ve
ARKELÉS onu üretemez.

**9.4** SQLite index türetilmiş veridir. Kaynak veri değildir. Silinirse
vault'tan yeniden kurulur.

## 10. Hızlı Yakalama İstisnası *(yeni madde)*

**Sorun:** Yeni görev oluşturmak semantik iştir (8.2), yalnız Hermes yapar.
Hermes kapalıysa kullanıcı aklına gelen bir şeyi hiçbir yere yazamaz.
Bir kişisel işletim sistemi için bu kabul edilemez.

**Çözüm:**

**10.1** ARKELÉS'in bir **Hızlı Yakalama** yüzeyi vardır (Cmd+K içinden erişilir).

**10.2** Hızlı Yakalama, vault içindeki tek bir **gelen kutusu** dosyasına
**ham bir satır ekler**. Başka hiçbir şey yapmaz: yapı kurmaz, sınıflandırmaz,
başlık atmaz, frontmatter yazmaz, yeni dosya açmaz.

**10.3** Satır eklemek mekanik bir işlemdir (8.1). Bu yüzden bu istisna
A3'ü ihlal etmez — ARKELÉS metne **anlam vermiyor**, yalnızca kaydediyor.

**10.4** Ham satıra anlam vermek, onu yapılandırılmış nota dönüştürmek
**Hermes'in işidir**. Hermes gelen kutusunu kendi zamanında işler.

**10.5** ARKELÉS gelen kutusunda bekleyen ham satır sayısını gösterebilir.
Onları yorumlayamaz.

## 11. Hayat Skoru Kuralı *(yeni madde — çelişki çözümü)*

**Sorun:** Dashboard "hayat skoru" gösterecek (madde 24). Ama skor bir **analizdir**
ve analiz Hermes'in tekelindedir (8.2). ARKELÉS skoru hesaplayamaz.

**Çözüm:**

**11.1** Hayat skorunu **Hermes hesaplar** ve vault'a yazar.

**11.2** ARKELÉS skoru **yalnızca okur ve gösterir**. Formülü bilmez, hesaplamaz.

**11.3** Skor, hesaplandığı zaman damgasıyla birlikte saklanır. ARKELÉS her zaman
**son bilinen skoru** gösterir ve tazeliğini belirtir. Hermes kapalıyken skor
kaybolmaz, sadece bayatlar.

**11.4** Aynı kural, ARKELÉS'in gösterdiği **her türev metrik** için geçerlidir:
öncelik sıralaması, risk göstergesi, ilerleme yüzdesi. Hepsi Hermes tarafından
üretilir, vault'ta saklanır, ARKELÉS tarafından okunur.

**11.5** İstisna: **saymak** analiz değildir. "3 görev açık", "2 not bugün değişti"
gibi doğrudan sayımlar index'ten hesaplanabilir. Sınır şudur — **sayı bilgi verir,
skor yargı verir.** ARKELÉS sayar, yargılamaz.

---

# BÖLÜM III — MİMARİ

## 12. Katmanlar

```
                    ARKELÉS (Tauri uygulaması)
┌───────────────────────────────────────────────────────────┐
│  WEBVIEW (React)                                          │
│   Sunum:  Katmanlar · Dock · Zoom Trail · Paneller        │
│   Durum:  Zustand (UI)  ·  TanStack Query (veri)          │
│           └─ tek IPC yüzeyi; component dosya sistemi      │
│              bilmez, Hermes'i tanımaz                     │
└──────────────────────────┬────────────────────────────────┘
                           │  Tauri IPC (tip güvenli)
┌──────────────────────────┴────────────────────────────────┐
│  ÇEKİRDEK (Rust)                                          │
│   Vault Okur (md parser)                                  │
│   Vault Yazar (yalnız mekanik · atomic · hash doğrulamalı)│
│   SQLite Index + FTS5   ← türetilmiş, her an silinebilir   │
│   Dosya İzleyici (notify · debounce)                      │
│   Hermes İstemcisi  ← token BURADA, webview'de DEĞİL      │
└──────────┬────────────────────────────────┬───────────────┘
           │ dosya sistemi                  │ 127.0.0.1 + token
    ┌──────┴───────┐                 ┌──────┴──────┐
    │ Obsidian     │◀──── yazar ─────│  HERMES     │
    │ Vault (.md)  │                 │  (daemon)   │
    │ = TEK GERÇEK │                 └─────────────┘
    └──────────────┘
```

## 13. Uzak Server Yok

**13.1** **Uzak** server yok. Bulut zorunluluğu yok. Hesap yok. Giriş ekranı yok.

**13.2** Hermes **lokal bir daemon'dur** ve mimarinin meşru parçasıdır.
Teknik olarak bir sunucu sunar; bu bir istisna değil, tasarımdır.

**13.3** MVP internetsiz tam çalışır.

**13.4** Uygulama, kullanıcının izni olmadan hiçbir dış adrese istek atmaz.
Telemetri yok. Çökme raporu gönderimi yok. Otomatik güncelleme kontrolü yok.

## 14. Teknoloji — Kesinleşmiş

| Katman | Teknoloji |
|---|---|
| Kabuk | **Tauri 2.x** |
| Arayüz | **React + TypeScript** |
| Stil | **TailwindCSS** + kendi token sistemimiz |
| Davranış primitifleri | **shadcn/ui** — yalnız madde 31 kapsamında |
| Hareket | **Framer Motion** (`LazyMotion` + `domAnimation`) |
| UI durumu | **Zustand** |
| Veri katmanı | **TanStack Query** |
| Çekirdek | **Rust** |
| Index | **SQLite** (`rusqlite`) + **FTS5** |
| Dosya izleme | `notify` |
| Tip üretimi | `ts-rs` (Rust → TypeScript) |

**14.1** Bu liste dışına çıkan her bağımlılık gerekçe ister. Gereksiz bağımlılık yasaktır.

**14.2** Elle yazılmış IPC tip tanımı yasaktır. Sözleşme Rust'ta yaşar, TypeScript'e üretilir.

## 15. Index *(A4)*

**15.1** Markdown dosyaları arayüz tarafından **doğrudan taranmaz**.

**15.2** Bütün okuma index üzerinden yapılır.

**15.3** Index Rust çekirdeğinde, **app-data dizininde** tutulur. Vault'un içinde **değil** —
vault kirlenmez.

**15.4** Vault → index akışı **tek yönlüdür**. Index'ten vault'a hiçbir zaman veri akmaz.

**15.5** Index şeması versiyonlanır. Sürüm değişince index otomatik yeniden kurulur.

**15.6** Dosya izleyici değişikliği yakalar, index'i artımlı günceller.
İçerik hash'i aynıysa yeniden indeksleme yapılmaz.

## 16. Kimlik (ID) Stratejisi

**16.1** Yönetilen her not, frontmatter'da **kalıcı bir `arkeles_id`** taşır (ULID).

**16.2** Dosya adı **insan içindir, kimlik değildir**. Dosya adı değişince
hiçbir ilişki kopmaz.

**16.3** `arkeles_id` taşımayan notlar **"yönetilmeyen"** sayılır: okunur, aranır,
gösterilir — ama ARKELÉS onlara mekanik mutasyon uygulamaz.

**16.4** Bir nota `arkeles_id` yazmak semantik bir işlemdir → **Hermes yapar**.

## 17. Vault Dışında Kalanlar

**17.1** Şunlar vault'a **yazılmaz**: pencere konumu ve boyutu, son açık katman,
dock sırası, panel açık/kapalı durumu, tema seçimi, index.

**17.2** Bunlar uygulama yapılandırmasıdır, bilgi değildir. Tauri app-config
dizininde tutulur.

**17.3** Vault yolu hiçbir yerde sabit kodlanmaz. İlk açılışta sorulur.

## 18. Hermes Sözleşmesi

**18.1** Hermes bir sürüm ve **yetenek listesi** yayınlar:

```
GET /health → { "version": "1.2.0", "capabilities": ["pdf", "telegram", "ai.summarize", ...] }
```

**18.2** ARKELÉS, Hermes'in bildirmediği bir yeteneği **arayüzde hiç göstermez**.
Tıklandığında hata veren buton yoktur.

**18.3 Hermes kapalıyken davranış — zorunlu tablo:**

| Yetenek | Hermes kapalı |
|---|---|
| Vault okuma, arama, gezinme | **Tam çalışır** |
| Mekanik mutasyonlar (8.1) | **Tam çalışır** |
| Hızlı Yakalama (10) | **Tam çalışır** |
| Son bilinen hayat skoru | **Görünür, tazelik belirtilir** |
| Semantik aksiyonlar (8.2) | Pasif. Sebebi sessizce belirtilir |
| Dock durum göstergesi | Gri |

**18.4** Hermes kapalı olması bir **hata durumu değildir**. Kırmızı uyarı,
modal, sesli bildirim veya tekrarlayan uyarı gösterilmez. Sadece gri bir göstergedir.

## 19. Güven Sınırı

**19.1** Hermes yalnızca `127.0.0.1`'e bind edilir. `0.0.0.0` yasaktır.

**19.2** Paylaşılan token dosyada `0600` izinle tutulur.

**19.3 Hermes'e webview'den asla doğrudan istek atılmaz.** Tüm çağrılar Rust
çekirdeği üzerinden geçer. **Token JavaScript'e hiç inmez.**

**19.4** Hermes, `Origin` header'ı taşıyan istekleri reddeder (tarayıcı kaynaklı
çağrı engellenir).

**19.5** Vault içeriği hiçbir log satırına yazılmaz. Log'da yalnızca ID ve dosya
yolu bulunur. Sağlık ve Finans verisi hassastır.

## 20. Yazma Güvenliği

**20.1 Atomic write zorunludur:** geçici dosyaya yaz → `fsync` → `rename`.
Yarım dosya asla görünmez.

**20.2 Optimistic concurrency zorunludur:** ARKELÉS yazarken, okuduğu andaki
`mtime + içerik hash`'ini doğrular.

**20.3** Uyuşmazlık varsa **yazmaz.** Kullanıcıya çakışma gösterir.
**Sessizce üzerine yazmak yasaktır.** Obsidian ve Hermes de aynı dosyalara yazıyor.

**20.4** Dosya izleyici event'leri 200 ms penceresinde birleştirilir (debounce).

**20.5** Senkronizasyon artefaktları yok sayılır: `.icloud`, `conflicted copy`,
`~$`, `dosya 2.md` desenleri.

**20.6** Vault bir **git repo** olarak tutulur. Bedava, bulut gerektirmez,
her arızada geri dönüş noktası verir.

---

# BÖLÜM IV — DENEYİM

## 21. Açılış Hissi

**21.1** Uygulama açıldığında kullanıcı **yeni bir program açmış gibi hissetmez.**
Kendi çalışma alanına girmiş gibi hisseder.

**21.2** Splash ekranı yok. Yükleniyor ekranı yok. Hoş geldin ekranı yok.
Tanıtım turu yok.

**21.3** Açılışta ilk görünen şey **son bilinen veri**dir; arkada tazelenir.
Kullanıcı boş bir iskelet ekranı beklemez.

**21.4** Kullanıcı uygulamayı kapattığı yerde bulur.

## 22. Navigasyon

**22.1** Sidebar **kullanılmaz.**

**22.2** **Bottom Dock** kullanılır.

**22.3** **Zoom Trail** kullanılır — kullanıcı nerede olduğunu her an bilir.

**22.4** **Layered Zoom Navigation** kullanılır.

**22.5** Zoom animasyonu **yalnızca ana katmanlarda** olur. En fazla 2 zoom seviyesi.

**22.6** Alt katmanlarda normal **panel geçişleri** kullanılır.

**22.7** Bütün modüller **aynı evren içindedir.** Hiçbir zaman yeni pencere hissi oluşmaz.
Yeni OS penceresi açılmaz. Modal yığını oluşturulmaz.

**22.8** Her gezinme **klavyeyle geri alınabilir** (`Esc`).

## 23. Hareket Anayasası

Tasarım kuralı **"hareket yerine hiyerarşi"**dir (madde 28). Bu, hareketin
yasaklanması değil, **hareketin işinin daraltılmasıdır.**

**23.1 Hareketin tek meşru işi mekânsal sürekliliktir.** Kullanıcıya nereden
nereye gittiğini anlatır. Başka hiçbir amaçla hareket kullanılmaz.

**23.2 Yasak hareket türleri:** dikkat çekme, dekorasyon, nefes alan/pulse eden
öğeler, süslü giriş animasyonları, sıralı (staggered) liste açılışları,
sürekli dönen/parlayan göstergeler.

**23.3** Yalnızca `transform` ve `opacity` animate edilir.
`width` / `height` / `top` / `left` / `filter` / `blur` büyük yüzeylerde **yasaktır**
(layout ve paint tetikler).

**23.4** Süreler: hızlı 120 ms · temel 200 ms · katman geçişi 260 ms.
**Üst sınır 300 ms.**

**23.5 Animasyon dekorasyondur, kapı değildir.** Hedef katman animasyonun
**1. karesinde etkileşime hazırdır.** Kullanıcı animasyon ortasında tıklarsa işlem geçer.

**23.6** Veri, animasyon **başlamadan önce** prefetch edilir. Zoom sırasında
spinner görülmez.

**23.7** Kaynak katman geçiş bitince **unmount edilir.** İki katman kalıcı mount kalmaz.

**23.8** `prefers-reduced-motion` desteklenir → süre 0. Uygulama tam çalışır.

## 24. Dashboard

**24.1** Dashboard yalnızca giriş ekranı değildir. **Her gün açılan ana çalışma ekranıdır.**
Sürekli kullanılacaktır.

**24.2** Tek bakışta göstermelidir:

- bugünkü durum
- kritik görevler
- Hermes durumu
- çalışılan kurumlar
- öncelikler
- hayat skoru (madde 11 kuralıyla)

**24.3 Tek bakış kuralı:** Bu bilgiler **kaydırmadan** görünür.
Kaydırma gerekiyorsa dashboard fazla şey gösteriyor demektir; bilgi eklenmez, çıkarılır.

**24.4** Dashboard bir grafik panosu değildir. Süs grafik, gereksiz sayaç,
dolan halka, gösterişli veri görselleştirmesi yoktur.

**24.5** Dashboard'daki her öğe **bir eyleme veya bir katmana** açılır.
Hiçbir öğe sadece durmak için durmaz.

## 25. Bildirim Doktrini *(yeni madde)*

Madde 5'teki "bildirim yağmuru oluşturmaz" ifadesinin uygulanabilir hali:

**25.1** ARKELÉS **sistem bildirimi göndermez.** Hiç. Bildirim işi Hermes'in
ve Telegram'ındır.

**25.2** ARKELÉS içinde bilgi **durum olarak** gösterilir, uyarı olarak değil.
Sayı, renk, gri gösterge — evet. Modal, toast yığını, kırmızı banner, ses — hayır.

**25.3** Kullanıcının aksiyonuna verilen geri bildirim **sessizdir**: öğe yerinde
değişir. "Kaydedildi" bildirimi gösterilmez — kullanıcı sonucu zaten görür.

**25.4** Yalnızca **veri kaybı riski** taşıyan durum kullanıcıyı bloke edebilir:
yazma çakışması (20.3). Bunun dışında hiçbir şey akışı kesmez.

**25.5** Aynı bilgi ikinci kez uyarılmaz. Tekrarlayan uyarı yasaktır.

## 26. Boş Durum Doktrini *(yeni madde)*

**26.1** Her ekranın boş durumu, dolu durumuyla **aynı sprintte** tasarlanır.
Sonradan eklenen boş durumlar hep çirkin kalır.

**26.2** Boş durum bir hata değildir. "Veri yok", "Hata", "Bulunamadı" gibi
soğuk metin kullanılmaz.

**26.3** Boş durum sakin, kısa ve varsa tek bir sonraki adımı gösterir.

**26.4** MVP'de hazır arayüz olarak duran modüller (madde 36) bilinçli boş
durumdadır — kırık değil, **henüz** boş.

## 27. Klavye *(A7)*

**27.1 Cmd+K komut paleti Sprint 1 kapsamındadır.** Sonraya bırakılamaz.

**27.2** Bir *işletim sistemi* hissini veren şey klavyedir. Dock keşif içindir;
günlük hız klavyeden gelir.

**27.3** Bütün aksiyonlar merkezi bir **aksiyon kaydından** (`actionRegistry`)
tanımlanır. Komut paleti bu kaydı okur. Kayıtta olmayan aksiyon yoktur.

**27.4** Cmd+K içinden erişilir: katmanlara gezinme, not arama (FTS5),
Hızlı Yakalama (madde 10), Hermes'in bildirdiği yetenekler.

**27.5** Fareyle yapılabilen her ana işlem klavyeyle de yapılabilir.

---

# BÖLÜM V — TASARIM

## 28. Tasarım İlkeleri

- Premium
- Minimal
- Sessiz
- Derinlik hissi
- Temiz boşluklar
- Büyük nefes alan kartlar
- Yumuşak gölgeler
- Cam efekti **gerektiği kadar** (madde 30)
- **Hareket yerine hiyerarşi** (madde 23)

**28.1** Referans tasarım daha sonra ekran görüntüsü olarak paylaşılacaktır.
O gelene kadar bu bölüm bağlayıcıdır. Referans geldiğinde bu bölüm güncellenir,
Bölüm I değişmez.

**28.2 Derinlik, gölge ve katman ile kurulur; hareket ile değil.**
Bir öğenin önemi, konumundan ve boşluğundan anlaşılır — animasyonundan değil.

## 29. Token Sistemi

**29.1** UI kodu yazılmadan önce `tokens.css` sabitlenir. Sonra tartışılmaz.
Token sistemi olmadan "premium" iddiası 3 sprintte çöker.

| Eksen | Değerler |
|---|---|
| Uzay | 4 · 8 · 12 · 16 · 24 · 32 · 48 · 64 |
| Radius | 6 · 10 · 16 |
| Tipografi | 11 · 13 · 15 · 20 · 28 · 40 (SF Pro / system-ui) |
| Yükseklik | 3 seviye — hiçbiri renkli gölge kullanmaz |
| Süre | `--dur-fast 120ms` · `--dur-base 200ms` · `--dur-layer 260ms` |
| Easing | `--ease-out cubic-bezier(.32,.72,0,1)` |
| Renk | Nötr gri ölçek + **tek** vurgu rengi |

**29.2** Karanlık tema varsayılandır. Aydınlık tema token seviyesinde desteklenir.

**29.3** Token dışında sabit değer (magic number) kullanılması kod incelemesinde
ret sebebidir.

## 30. Cam Efekti Kuralı *(çelişki çözümü)*

**Sorun:** Cam efekti (`backdrop-filter`) tasarım dilinin parçası (madde 28) ama
`filter`/`blur` performans açısından en pahalı CSS özelliğidir ve madde 23.3
büyük yüzeylerde yasaklıyor. İkisi doğrudan çelişiyor.

**Çözüm — cam efekti kullanılabilir, ama üç koşulla:**

**30.1 Küçük ve sınırlı yüzeylerde:** dock, komut paleti, üst şerit, açılır panel.
Tam ekran arka planlarda, uzun kaydırma alanlarında, kart listelerinde **yasak**.

**30.2 Asla animate edilmez.** Cam bir yüzey ya vardır ya yoktur.
`backdrop-filter` değeri geçiş sırasında değişmez. Zoom animasyonu boyunca
cam yüzey ya sahnede değildir ya sabittir.

**30.3 Katman sayısı sınırlıdır.** Aynı anda ekranda **en fazla 2** cam yüzey bulunur.
Üst üste binen cam yüzey yoktur.

**30.4** Cam efekti performans bütçesini (madde 34) aşarsa **düşer**, bütçe düşmez.
Bütçe pazarlık konusu değildir; efekt konusudur.

## 31. Bileşen Kütüphanesi Kapsamı *(A5)*

**31.1** shadcn/ui **yalnızca davranış primitifleri** için kullanılır:
`Dialog`, `Popover`, `Command`, `Tooltip`, `ScrollArea`.

**31.2 Gerekçe:** Bunların focus yönetimi, klavye davranışı ve erişilebilirlik
işini sıfırdan yazmak haftalar alır ve hatalı olur.

**31.3 Görsel katman tamamen ARKELÉS'e özgüdür.** shadcn'in varsayılan
renk / gölge / radius teması **kullanılmaz**; `tokens.css` üzerine bindirilir.

**31.4 Gerekçe:** shadcn'in varsayılan görünümü, AI ile üretilmiş her dashboard'un
görünümüdür. "Apple seviyesinde premium, sessiz" hedefiyle doğrudan çelişir.

## 32. Yasaklar

Aşağıdakiler tasarım tartışmasına kapalıdır:

- Neon
- Gradyan arka plan
- Glow / parlama efekti
- Renkli gölge
- Cyberpunk / bilim kurgu göstergeleri
- HUD çerçeveleri, tarama çizgileri, "teknolojik" süsler
- Emoji ikon
- Süs grafik ve dolan halka
- Karşılama / tanıtım ekranı
- Toast yığını
- 3 seviyeden derin gölge
- Yazı tipi karnavalı (tek aile: system-ui)

---

# BÖLÜM VI — PERFORMANS

## 33. Sakinlik ≠ Yavaşlık *(çelişki çözümü)*

**Sorun:** Madde 5 "acele etmez, sakin çalışır" diyor. Prompt 1 "hız her şeyden
önemlidir" diyor. Görünürde çelişki var.

**Çözüm:**

**33.1 Sistem hızlıdır. Arayüz sakindir.**

**33.2** *Sakin* tondur: bağırmaz, uyarı yağdırmaz, dikkat çalmaz.
*Hızlı* gecikmedir: hiçbir işlem beklemez.

**33.3** Bunlar çelişmez, birbirini gerektirir. **Yavaşlık sakinlik değildir, kabalıktır.**
Kullanıcıyı bekleten bir arayüz sessiz olamaz.

**33.4** "Sakin" hiçbir zaman yapay gecikme, uzatılmış animasyon veya
"düşünüyor" hissi vermek için kullanılmaz.

## 34. Performans Bütçesi

**34.1** Bu bütçe **sert alt sınırdır.** Deneyim uğruna esnetilmez —
çünkü bütçe ihlali zaten kötü deneyimdir (33.3).

| Metrik | Bütçe |
|---|---|
| Soğuk açılış → ilk anlamlı kare | < 800 ms |
| Katman geçişi (p95 kare süresi) | < 8,3 ms (120 Hz) |
| Etkileşim → görsel tepki | < 100 ms |
| Bugün görünümü sorgusu (2.000 not) | < 10 ms |
| Tam index kurulumu (2.000 not) | < 1 sn |
| Boşta RAM | < 180 MB |
| JS bundle (gzip) | < 250 KB |

**34.2** Bütçe her sprint sonunda **ölçülür ve raporlanır.**
Bütçeyi aşan sprint kapanmaz.

**34.3** Bir özellik bütçeyi aşıyorsa, özellik düşer. Bütçe düşmez.

## 35. Render Kuralları

**35.1 Gereksiz render yasaktır.**

**35.2** Sıcak veri React Context'te tutulmaz. Zustand'ın selector'lı aboneliği
kullanılır — yalnızca ilgili component render olur.

**35.3** Liste öğeleri `React.memo` + kararlı `key` kullanır.
200+ satırlı listeler sanallaştırılır.

**35.4** Her sprint sonunda React DevTools "highlight updates" ile görsel doğrulama yapılır.

**35.5** Native his korunur. Uygulama bir web sitesi gibi davranmaz:
metin seçimi kontrollüdür, sürükleme alanları tanımlıdır,
tarayıcı kaydırma davranışı taşmaz.

---

# BÖLÜM VII — MODÜLLER

## 36. Modül Listesi ve MVP Kapsamı *(A6)*

**36.1** Dock **bütün ana modülleri gösterir.** Zihinsel model baştan bütündür.

**36.2** MVP boyunca yalnızca gerekli modüller **gerçek veriyle** çalışır.
Diğerleri hazır arayüz olarak bulunur (madde 26.4).

| Modül | MVP durumu |
|---|---|
| **Bugün** | Gerçek veri |
| **İş** | Gerçek veri — 2 çalışma alanı |
| **Sistem** | Gerçek veri |
| Kişisel | Hazır arayüz |
| Sağlık | Hazır arayüz |
| Finans | Hazır arayüz |
| Öğrenme | Hazır arayüz |
| İçerik | Hazır arayüz |
| Sosyal | Hazır arayüz |

**36.3** İş modülü altındaki çalışma alanları:
WIF · GEN · TüGA · Burkon · Merci · KEPDER · Öğrenciyiz.biz.tr

**36.4** MVP'de bunlardan **2'si** gerçek veriyle çalışır; hangileri Sprint 3'te seçilir.

**36.5 Gerekçe:** Yarım yapılmış 16 yüzey, tam yapılmış 3 yüzeyden kötüdür.
"Önce sağlam temel oluştur" kuralının doğal sonucudur.

---

# BÖLÜM VIII — GELİŞTİRME

## 37. Karar Sırası

Teknik karar verirken sıra:

1. **Ürün deneyimi**
2. **Performans**
3. **Kod sadeliği**
4. **Geliştirici kolaylığı**

**37.1 Önemli nüans:** Performans, deneyimin **rakibi değil bileşenidir.**
Madde 34 bütçesi bu yüzden 1. sıraya tabi değildir — bütçeyi ihlal etmek
zaten deneyimi ihlal etmektir (33.3).

**37.2** Sıra 1 ile 2 arasında gerçek bir seçim çıkarsa: deneyim kazanır,
**ama yalnızca bütçe içinde kalıyorsa.** Bütçe dışı deneyim önerisi reddedilir.

**37.3** "Geliştirici kolaylığı" en sondadır. Kolay olduğu için seçilen
kütüphane, mimari veya kısayol gerekçe kabul edilmez.

## 38. Sprint Disiplini

**38.1** Tek seferde uygulama geliştirilmez. Sprint mantığıyla ilerlenir.

**38.2** Her sprint sonunda **çalışan bir uygulama** oluşur. Yarım bırakılmış
sprint yoktur.

**38.3** Riskli ve belirsiz parça sprint içinde **önce** yapılır.
Kolay UI işi sona bırakılır — tersi iki kez iş yapmak demektir.

**38.4** Sprint kapanışı üç şeyi gerektirir: bitti tanımının tamamı,
performans bütçesi ölçümü (34.2), testlerin geçmesi.

## 39. Teknik Borç ve Test

**39.1 Teknik borç oluşturulmaz.** Bu bir dilek değil, sprint kapanış koşuludur.

**39.2** Modüler geliştirilir. Bir modülün silinmesi diğerlerini bozmaz.

**39.3 Zorunlu test kapsamı:** Rust markdown parser'ı ve index katmanı.
En riskli ve en sessiz bozulan yer burasıdır.
Test vakaları: bozuk YAML, eksik frontmatter, boş dosya, BOM, CRLF, emoji, 5 MB not.

**39.4** Saf mantık Vitest ile test edilir. E2E test MVP'de yoktur —
tek kişilik geliştirmede maliyeti faydasını aşar.

**39.5** Vault'un bir kopyası test fixture'ı olarak tutulur.
Gerçek vault üzerinde test çalıştırılmaz.

## 40. Değişiklik Yönetimi

**40.1** Bölüm I değiştirilemez.

**40.2** Bölüm III ve VI, gerekçeli öneriyle değiştirilebilir.
Değişiklik `docs/ADR/NNN-baslik.md` altında karar kaydı olarak yazılır:
bağlam, karar, gerekçe, reddedilen alternatifler.

**40.3** Yeni öneri sunulurken madde 6.2 sınaması yapılır:
*Bu, ürün kimliğini bozar mı?* Bozuyorsa teknik olarak mümkün olsa bile sunulmaz.

**40.4** Bu anayasanın sürüm numarası, her değişiklikte artar.
Değişiklik geçmişi git'te yaşar.

---

# EK A — Çözülen Çelişkiler

İki promptu birleştirirken bulunan gerçek çelişkiler ve çözümleri.
Felsefe hiçbirinde değiştirilmedi.

| # | Çelişki | Çözüm | Madde |
|---|---|---|---|
| 1 | "Server yok" ↔ Hermes bir Local API sunuyor | **Uzak** server yok; Hermes lokal daemon'dur ve mimarinin meşru parçası | 13 |
| 2 | "ARKELÉS veri üretmez" ↔ SQLite index onaylandı | *Yeni bilgi* üretmez; *türetilmiş* veri üretebilir. Silme testi (9.3) sınırı çiziyor | 9 |
| 3 | Yeni görev oluşturma Hermes'in ↔ Hermes kapalıyken hiçbir şey kaydedilemez | Hızlı Yakalama: ARKELÉS ham satır ekler (mekanik), Hermes anlam verir (semantik) | 10 |
| 4 | Dashboard "hayat skoru" gösterecek ↔ analiz Hermes tekelinde | Hermes hesaplar ve vault'a yazar; ARKELÉS son bilinen değeri okur. Sayma serbest, yargı değil | 11 |
| 5 | "Hareket yerine hiyerarşi" ↔ Zoom Trail hareket temelli | Hareketin tek işi mekânsal süreklilik. Dekoratif hareket yasak | 23 |
| 6 | "Cam efekti" ↔ `blur` performans bütçesini tehdit ediyor | Cam serbest ama küçük yüzeyde, animate edilmeden, en fazla 2 katman | 30 |
| 7 | "Sakin çalışır, acele etmez" ↔ "hız her şeyden önemli" | Sistem hızlı, arayüz sakin. Yavaşlık sakinlik değil, kabalıktır | 33 |
| 8 | Karar sırası deneyim > performans ↔ performans bütçesi sert sınır | Performans deneyimin bileşeni, rakibi değil. Bütçe ihlali = deneyim ihlali | 37.1 |

---

# EK B — Yol Haritası

**Sprint 0 — Temel** *(yarım gün)*
Rust toolchain kurulumu · Tauri 2 + React + TS iskeleti · `tokens.css` yerleşimi ·
vault yolu yapılandırması · `git init` · ADR klasörü

**Sprint 1 — Kabuk ve Çekirdek** *(5–7 gün)*
Rust: markdown parser · SQLite index + FTS5 · dosya izleyici · Hermes `/health`
Arayüz: token sistemi · katman motoru · Bottom Dock · Zoom Trail · Cmd+K ·
`Bugün` modülü (salt-okunur)

**Sprint 2 — Yazma**
Mekanik mutasyonlar · atomic write · çakışma arayüzü · optimistic update ·
Hızlı Yakalama

**Sprint 3 — İş Modülü**
2 çalışma alanı · alt panel geçişleri · ilişkili not görünümü

**Sprint 4 — Hermes Komutları**
Semantik aksiyonlar · iş kuyruğu · ilerleme durumu · hayat skoru gösterimi

**Sprint 5 — Genişleme**
Kalan modüller · sağlamlaştırma

---

**Bu anayasa yürürlüktedir. Geliştirme bundan sonra bu dokümana göre yapılır.**
