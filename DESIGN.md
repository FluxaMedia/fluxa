# Fluxa Desktop — Tasarım Dili

Her ekran aynı sözlükten konuşur. Yeni bir değer uydurmak yerine buradaki basamaklardan birini seç; hiçbiri uymuyorsa önce `src/design/tokens.ts`'i tartış, sonra ekranı yaz.

## Kaynak

- `src/design/tokens.ts` — renk, ölçek, boşluk, tipografi, hareket, katman
- `src/design/primitives.tsx` — `Button`, `IconButton`, `Chip`, `SectionLabel`, `Divider`, `MetaText`
- `src/index.css` — `@font-face` tanımları ve primitiflerin hover durumları

İsimlendirme, Android tarafındaki `FluxaColors` / `FluxaDimensions` yapısını yansıtır. İki repo kodu paylaşmıyor, sözlüğü paylaşıyor.

## Kurallar

**Ham renk yazma.** JSX veya stil nesnelerinde `#RRGGBB` / `rgba()` geçmez. `color.textMuted` yaz, `rgba(255,255,255,0.55)` yazma. Tek istisna `src/design/` ve `src/index.css`.

**Ölçek dışına çıkma.** `borderRadius`, `fontSize` ve boşluk değerleri `radius`, `fontSize`, `space`'ten gelir. `0.8438rem` gibi bir değer yazıyorsan bu bir karar değil, kazadır.

**Başlıklar Archivo, gövde Montserrat.** Başlık için `heading('h1')` yardımcısını kullan — ağırlık, genişlik ekseni ve harf aralığı birlikte gelir. Gövde metni `font.body` kalır.

**Katman numarası uydurma.** `z.overlay`, `z.dialog` gibi rollerden seç. `zIndex: 99999` yazmak, bir sonraki kişinin `999999` yazmasına davetiye.

**Yeni buton çizme.** Bir yerde daire içinde ikon lazımsa `IconButton`, seçilebilir etiket lazımsa `Chip`. Kendi `<button style={{...}}>`'unu yazıyorsan primitif eksik demektir — primitifi genişlet.

## Renk rolleri

| Rol | Kullanım |
|---|---|
| `bg` | Uygulama zemini |
| `bgElevated` | Detay/hero zemini, backdrop'un oturduğu kat |
| `surface` / `surfaceRaised` | Kart ve panel yüzeyleri |
| `textPrimary` → `textFaint` | Metin hiyerarşisi; dördünden fazlası yok |
| `line` / `lineStrong` | Ayırıcılar ve kenarlıklar |
| `fill` / `fillHover` / `fillActive` | Etkileşimli yüzeylerin üç durumu |
| `accent` | Marka turuncusu — ilerleme, seçim işareti, vurgu. Zemin veya buton dolgusu değil |

Birincil aksiyon **beyaz** düğmedir, accent değil. Accent bir durumu işaretler, bir şeyi süslemez.

## Tipografi ölçeği

`micro` 0.625 · `xs` 0.6875 · `sm` 0.75 · `base` 0.8125 · `md` 0.875 · `lg` 1 · `xl` 1.125 · `xxl` 1.375 · `h1` 2 · `hero` 3.125 (rem)

Arayüz metninin varsayılanı `base`. `lg` ve üzeri başlık demektir.

## Kontrol

```
npm run lint:design
```

`src/design/` dışında ham renk, ölçek dışı `borderRadius`/`fontSize` ve serbest `zIndex` arar. Kontrol, göç etmiş dosyalar için çalışır — liste `scripts/check-design.mjs` içindeki `MIGRATED`'dır. Bir ekranı token'lara taşıdıkça listeye ekle; liste büyüdükçe kontrol sıkılaşır.

## Göç durumu

- [ ] `src/components/detail/**`
- [ ] `src/screens/HomeScreen.tsx`
- [ ] `src/screens/LibraryScreen.tsx`
- [ ] `src/screens/SearchScreen.tsx`
- [ ] `src/screens/DiscoverScreen.tsx`
- [ ] `src/screens/CategoryGridScreen.tsx`
- [ ] `src/screens/SettingsScreen.tsx` + `src/components/settings/**`
- [ ] `src/screens/CalendarScreen.tsx`
- [ ] `src/screens/welcome/**`, `src/screens/Profile*`
- [ ] `src/components/player/**`
