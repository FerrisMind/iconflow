📊 РЕШЕНИЕ: **iconflow** (финальная документация для handoff) | ✅Выбор: 1 crate + Cargo feature-флаги, GUI-agnostic API “иконка = (Font Family + Unicode codepoint)”, **единый fallible string API** (`try_icon` → `Result`), генерация кода *предварительно* и коммит в репозиторий | 🔍Источники (3-5): Cargo features [1] · Conditional compilation (`cfg`) [2] · iced `Font` API [3] · egui `FontData::from_static` · JSON Schema Draft 2020-12 [4] · OWASP A08 integrity failures [5] | 💡Обоснование: разработчики GUI получают “как обычные иконки” через механизм шрифтов в GUI, а размер/состав паков контролируется feature-флагами. [1][2]

## Executive Summary
`iconflow` — Rust-crate, который поставляет наборы иконок в формате TTF и даёт единый способ получить **FontAsset** (байты шрифта) и **IconRef** (family+codepoint) для рендера в любых GUI (приоритет: iced/egui).[3]
Подключение конкретных иконпаков/вариантов делается Cargo feature-флагами и `cfg`, чтобы пользователи не тянули лишние ассеты.[1][2]
Адаптеры под GUI **не входят** в crate: пользователь сам регистрирует шрифты и рисует глифы.

## Требования и UX
### Целевая аудитория
- Rust GUI-разработчики (особенно iced/egui), которые хотят использовать известные иконпаки без статичных SVG/PNG.

### Ключевые требования (Must)
- 1 crate, пакеты включаются feature-флагами.[1]
- Размеры: `tiny/mini/regular/custom(u16)`.  
- Стили: единый нормализованный набор (outline/filled/regular/…), при этом внутри вы уже привели шрифты к единому формату.  
- “Максимально просто”: разработчик подключает пак, регистрирует `fonts()`, и дальше использует `IconRef` (codepoint) как обычный символ шрифта.

### Правила удобства (решения)
- Имена иконок **pack-local** (без глобальных “канонических” алиасов), чтобы избежать конфликтов и спорной семантики между наборами.  
- **Один публичный режим API:** строковый lookup через `try_icon(pack, name, style, size) -> Result<IconRef, IconError>`. Неизвестное имя или недоступный вариант → `Err` (не panic). Автокомплит по именам — через `list(pack)` и IDE/`&str`, не через публичные per-pack enum’ы.

## Технический дизайн
### Публичные типы (контракт)
```rust
pub enum Size { Tiny, Mini, Regular, Large, Custom(u16) }

pub enum Style {
    Regular, Filled, Outline, Light, Thin, Bold, Duotone, Glyph, Sharp, Rounded,
}

pub struct FontAsset {
    pub family: &'static str,
    pub bytes: &'static [u8],
}

pub struct IconRef {
    pub family: &'static str,
    pub codepoint: u32,
}

pub enum IconError {
    PackDisabled { pack: &'static str },
    IconNotFound { pack: &'static str, name: Cow<'static, str> },
    VariantUnavailable {
        pack: &'static str,
        name: Cow<'static, str>,
        requested: (Style, Size),
        available: &'static [(Style, Size)],
    },
}
```

`IconError` is `#[non_exhaustive]`. Fields named `name` use `std::borrow::Cow<'static, str>` (borrowed from pack tables on variant miss; owned when the lookup string was not found).

`Pack` is re-exported at the crate root (variants gated by `pack-*` features). The Fluent UI variant is **`Pack::FluentUi`** (feature `pack-fluentui`).

### Публичные функции (core)
```rust
pub fn fonts() -> &'static [FontAsset];

pub fn try_icon(pack: Pack, name: &str, style: Style, size: Size)
    -> Result<IconRef, IconError>;

pub fn list(pack: Pack) -> &'static [&'static str];
```

### Typed pack enums
**Not part of the public surface.** Generated pack modules live under `crate::generated` (`pub(crate)` only). There is no public infallible `Icon` / `LucideIcon` / `PhosphorIcon` API. Consumers use `try_icon` / `list` / `fonts` and the root-reexported `Pack` enum.

### Интеграция в GUI (ответственность пользователя)
- **egui**: шрифт регистрируется через `FontDefinitions`; байты передаются как `Arc::new(FontData::from_static(...))` (см. `docs/quickstart.md` / demo на `eframe::egui`).   
- **iced**: при выборе шрифта используется структура `Font` (в т.ч. семейство/стиль/вес), поэтому `FontAsset.family` должен быть корректным и стабильным для конкретного TTF.[3]
- Важно (iced): `family` в iconflow — это **Font Family внутри TTF**, а не имя файла (иначе пользователи будут получать “шрифт не применяется”).[6]

### Feature-флаги (Cargo.toml)
Feature-флаги — основной механизм контроля размера и состава сборки.[2][1]

| Категория | Пример фич | Что включает |
|---|---|---|
| Пакеты | `pack-lucide`, `pack-phosphor`, `pack-heroicons`, `pack-fluentui` | Иконки + варианты пакета |
| Размерные наборы | `heroicons-tiny/mini`, `octicons-tiny` | Доп. TTF под размеры (regular идёт с `pack-*`) |
| Convenience | `all-packs` (не default) | Для demo/внутренней проверки |

Рекомендация: `default = []`, чтобы по умолчанию ничего не тянуть.[1]

GUI toolkits (egui/iced) — только `[dev-dependencies]` для примеров; **не** crate features.

## Данные и генерация
### Репозиторий (ожидаемая структура)
- `assets/fonts/<pack>/<variant>.ttf` — ваши TTF (лицензии MIT/Apache-2.0).  
- `assets/maps/<pack>.json` — pack-local список иконок + варианты.  
- `assets/schema/iconflow-pack.schema.json` — единая JSON-schema.[4]
- `src/generated/**` — сгенерированные модули (таблицы lookup), **закоммичены**.  
- `xtask/` — генератор `gen`.

Почему генерация коммитится: чтобы потребитель крейта не зависел от build-скриптов и внешних тулов, а сборка была воспроизводимой (генерация — задача разработчиков iconflow, не пользователей).  

### JSON Schema (Draft 2020-12)
Использовать Draft 2020-12 как базу для валидации `assets/maps/*.json`.[4]

Минимальный формат (обязательные поля, `codepoint` — Unicode scalar без surrogate-диапазона):
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "iconflow-pack",
  "type": "object",
  "required": ["pack_id", "variants", "icons"],
  "properties": {
    "pack_id": { "type": "string" },
    "variants": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "style", "size", "family", "ttf_asset_path"],
        "properties": {
          "id": { "type": "string" },
          "style": { "type": "string" },
          "size": { "type": "string" },
          "family": { "type": "string" },
          "ttf_asset_path": { "type": "string" }
        }
      }
    },
    "icons": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name"],
        "properties": {
          "name": { "type": "string" },
          "codepoint": { "type": "integer", "minimum": 0, "maximum": 1114111 },
          "overrides": { "type": "object" },
          "availability": { "type": "array", "items": { "type": "string" } }
        }
      }
    }
  }
}
```

### Генератор (поведение)
- Валидирует JSON по schema.[4]
- Генерирует:
  - sorted `IconEntry` / `ICON_NAMES` таблицы для binary-search lookup по `name`.
  - таблицы `name -> codepoint` с учётом `overrides` по variant.
  - таблицу `VariantKey(style,size) -> FontAsset`.
- Инварианты (в тестах генерации): имена отсортированы, `ICON_NAMES.len() == ICON_ENTRIES.len()`, каждый entry имеет хотя бы один валидный `VariantKey` (иначе генерация падает).
- Публичный `enum Icon` / infallible typed API **не** эмитится.

## План, качество, риски
### MVP / Beta / Release
- MVP: 3–5 паков, `fonts()/try_icon()/list()`, генератор, README с iced+egui рецептом.[3]
- Beta: все паки из `fonts/`, строгая диагностика `VariantUnavailable { available }`.  
- Release: semver-стабилизация, CI по фичам паков, контроль целостности ассетов (см. ниже).[1]

### DoD (Definition of Done)
- ✅ `cargo test` проходит при `--no-default-features` и для каждого `--features pack-*`.[1]
- ✅ Все public API задокументированы rustdoc + есть 2 примера (egui/iced) регистрации `fonts()`.[3]
- ✅ Генератор детерминированный (одинаковый вход → одинаковый `src/generated`).  
- ✅ Никаких обязательных GUI-зависимостей в `iconflow` core.

### Risk matrix (likelihood × severity)
OWASP относит проблемы целостности/подмены компонентов к классу integrity failures, поэтому артефакты шрифтов и генерация должны быть защищены процессом.[5]

| Риск | Likelihood | Severity | Mitigation |
|---|---:|---:|---|
| Раздувание размера бинарника из-за TTF | 4 | 3 | Фичи по пакам/вариантам. [1] |
| iced: неверный `family` → “иконка не тем шрифтом” | 3 | 4 | `family` берётся из TTF, фиксируется в map; примеры в README. [6] |
| Подмена/порча ассетов | 2 | 4 | Хэши TTF в репо/CI + SBOM/аудит как доп. меры. [5] |

***

[1](https://doc.rust-lang.org/cargo/reference/features.html)
[2](https://doc.rust-lang.org/reference/conditional-compilation.html)
[3](https://crates.io/crates/icons)
[4](https://json-schema.org/draft/2020-12)
[5](https://owasp.org/Top10/2025/A08_2025-Software_or_Data_Integrity_Failures/)
[6](https://github.com/iced-rs/iced/discussions/1988)
