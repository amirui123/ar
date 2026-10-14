# 🎯 ۱۲ — ۱۰۰ ترفند Rust (مرجع سریع)

> هر ترفند با یک قطعهٔ کوتاه و یک «چرا». شماره‌ها را در کد/گفتگو ارجاع بده.
> ۱۲ ترفند اولِ متن اصلی README، ترفندهای همین پروژه‌اند؛ اینجا ۱۰۰ تای کامل است.

---

## الف) مالکیت و قرض (#۱–#۱۶)

**#۱ نسبت دادن بدون کپی:** `let b = a;` برای انواع `Copy` کپی است و برای بقیه
«انتقال». برای کپی صریح: `a.clone()`.
```rust
let s1 = String::from("x");
let s2 = s1;                 // s1 دیگر معتبر نیست (انتقال)
println!("{s2}");
```

**#۲ `&` در برابر `&mut`:** هر تعداد `&` ولی فقط **یکی** `&mut` در هر زمان —
و اگر `&mut` هست، هیچ `&` دیگری هم نمی‌تواند باشد.

**#۳ NLL (عمر غیرواژگانی):** قرض همان‌جایی که آخرین بار استفاده شده تمام
می‌شود، نه آخر بلوک.
```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
println!("{first}");     // ← آخرین استفاده
v.push(4);               // ✅ چون first دیگر زنده نیست
```

**#۴ `split_at_mut` برای دو قرض تغییرپذیر:**
```rust
let (a, b) = v.split_at_mut(2);      // a و b همپوشانی ندارند ✅
a[0] = 9; b[0] = 8;
```

**#۵ کپی‌کردن از پشت مرجع:** وقتی فقط `&T` داری ولی مقدار را لازم داری →
`T: Copy` باشد (`*r`) یا `r.clone()`.

**#۶ `mem::take` / `mem::replace`:** برداشتن مقدار از داخل `&mut` بدون
داشتن مقدار جایگزین.
```rust
let mut v = vec![1, 2, 3];
let old = std::mem::take(&mut v);     // v خالی می‌شود، old مالک داده‌هاست
```

**#۷ `std::mem::swap`:** جابه‌جایی دو مقدار.
```rust
std::mem::swap(&mut a, &mut b);
```

**#۸ `Cow`:** صفر تخصیص در مسیر رایج (بخش ۰۷ #۸).

**#۹ `'static` در برابر `&'static`:** `T: 'static` یعنی «هیچ مرجع کوتاه‌عمری
داخلش نیست» — نه «همیشه زنده است». هر `String` مالک، `'static` است.

**#۱۰ ترفند عمر: «عمر خروجی = عمر یکی از ورودی‌ها»**
```rust
fn first<'a>(x: &'a str, y: &str) -> &'a str { x }   // y بی‌ربط است
```

**#۱۱ در ساختارها، عمر را همان‌جا اعلام کن**
```rust
struct Parser<'a> { input: &'a str, pos: usize }
```

**#۱۲ `'_` برای عمر ناشناخته (elided):** `fn f(x: &str) -> Cow<'_, str>`.

**#۱۳ ترفند: برای «مرجع یا مالکیت» از `impl Into<String>` یا `Cow` استفاده کن**
نه `&str` — API انعطاف‌پذیر می‌شود.

**#۱۴ `drop(x)` برای آزادسازی زودهنگام** (قفل، فایل، حافظه).
```rust
{ let _guard = m.lock().unwrap(); }   // ← همین بلوک = عمر قفل
```

**#۱۵ `ManuallyDrop`:** جلوگیری از اجرای `Drop` (FFI، `union`، union-like).

**#۱۶ ترفند `#[repr(transparent)]`:** newtype بدون هیچ هزینه‌ای در ABI
(همان کاری که `TaskId` ما می‌کند):
```rust
#[repr(transparent)]
pub struct TaskId(u64);       // در حافظه دقیقاً یک u64 است
```

---

## ب) انواع و traitها (#۱۷–#۳۴)

**#۱۷ `#[derive]` هوشمند:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Default,
PartialOrd, Ord` — همیشه فقط چیزهایی که معنا دارند.

**#۱۸ `Default` برای «راحت‌سازی ساختن»:**
```rust
let cfg = Config { port: 8080, ..Default::default() };
```

**#۱۹ الگوی Builder:** برای ساختارهای با فیلد زیاد:
```rust
let req = Request::builder().method("GET").path("/x").build();
```

**#۲۰ `impl From<A> for B` یک `Into` مجانی می‌دهد:** پس فقط `From` بنویس.
```rust
impl From<Task> for TaskRow { fn from(t: Task) -> Self { … } }
let row: TaskRow = task.into();
```

**#۲۱ `?` روی `Option` و `Result` هر دو کار می‌کند** — در تابعی با نوع
بازگشتی `Option<T>` می‌توانی `?` بزنی.

**#۲۲ trait object در برابر جنریک:**

| | `fn f(x: &dyn Trait)` | `fn f<T: Trait>(x: &T)` |
|---|---|---|
| تصمیم | زمان اجرا | زمان کامپایل (monomorphization) |
| حجم کد | کم | زیاد (برای هر نوع یک نسخه) |
| سرعت | پویا (vtable) | درون‌خطی‌شدنی |
| ناهمگون (`Vec<Box<dyn>>`) | ✅ | ❌ |

**#۲۳ `dyn` + auto trait:** `Box<dyn Error + Send + Sync>` — دام‌ها را با
`+` جمع کن (همان چیزی که در `web.rs` لازم شد).

**#۲۴ `?Sized`:** برای تابعی که هم `str`/`[T]`/`dyn Trait` بگیرد:
```rust
fn describe<S: Shape + ?Sized>(s: &S) -> String { s.name().into() }
```

**#۲۵ trait با متد پیش‌فرض:** پیاده‌سازی پیش‌فرض، سازگاری پذیری می‌سازد.

**#۲۶ trait سفارشی برای منع پیاده‌سازی بیرونی (Sealed trait):**
```rust
mod private { pub trait Sealed {} }
pub trait MyApi: private::Sealed { fn go(&self); }   // ← بیرونی‌ها نمی‌توانند impl کنند
```

**#۲۷ `impl Trait` در جایگاه آرگومان و بازگشتی:**
```rust
fn to_iter(x: &[i32]) -> impl Iterator<Item = i32> + '_ { x.iter().map(|v| v * 2) }
```
⚠️ بازگشتی: فقط یک نوع مشخص (برای چند نوع → `Box<dyn Iterator>`).

**#۲۸ `AsRef<Path>` برای مسیرها:**
```rust
fn load(p: impl AsRef<std::path::Path>) { let p = p.as_ref(); … }
load("a.txt"); load(PathBuf::from("b.txt"));
```

**#۲۹ `IntoIterator` برای ساختار خودت:** تا `for x in my_thing {}` کار کند.

**#۳۰ `FromIterator`:** تا `my_thing.collect::<MyThing>()` کار کند.

**#۳۱ `Borrow<str>` برای `HashMap`:** کلید `String` + جست‌وجو با `&str`
بدون تخصیص:
```rust
map.get("key")     // ✅ چون String: Borrow<str>
```

**#۳۲ `PhantomData`:** نگه‌داشتن پارامتر نوع بدون استفادهٔ واقعی (علامت مالکیت/تگ).
```rust
struct Id<T> { raw: u64, _marker: PhantomData<T> }
```

**#۳۳ `enum` به‌جای bool برای خوانایی:** `Mode::DryRun` بهتر از `true` است.

**#۳۴ `#[non_exhaustive]`:** افزودن واریانت جدید بدون شکستن کد کاربران.

---

## ج) رشته و متن (#۳۵–#۴۶)

**#۳۵ `String` = `Vec<u8>` + تضمین UTF-8.** اندیس `s[0]` ممنوع است!
به‌جایش `s.chars().nth(0)`, `s.as_bytes()[0]`, یا `s.get(0..1)`.

**#۳۶ مرز کاراکتر:** `&s[i..j]` اگر وسط یک کاراکتر چندبایتی بیفتد **پنیک**
می‌کند (بلای فارسی/عربی!). راه امن:
```rust
s.get(i..j).unwrap_or("")             // ← الگویی که در lifetimes.rs استفاده کردیم
s.is_char_boundary(i)
s.char_indices()                       // (byte_index, char)
```

**#۳۷ طول‌ها:** `s.len()` = **بایت** · `s.chars().count()` = کاراکتر ·
`s.encode_utf16().count()` = واحد UTF-16.

**#۳۸ `format!` ماکروی همه‌کاره:**
```rust
format!("{name:>8}")   // راست‌چین با عرض ۸
format!("{n:05}")      // صفر پُر
format!("{n:+}")       // علامت همیشه
format!("{f:.2}")      // دو رقم اعشار
format!("{b:#b}")      // باینری با 0b
format!("{n:#010x}")   // هگز با صفر
```
⚠️ **`format!` جداکننده هزارگان (`{:,}`) ندارد** — باید دستی بنویسی
(`group_digits` در `strings.rs`).

**#۳۹ `str::split_whitespace`:** سریع‌ترین راه تقسیم روی هر فضای خالی (شامل
تب و خط جدید و فاصلهٔ فارسی).

**#۴۰ `trim_matches(['"', '\''])`:** حذف چند کاراکتر از دو طرف.

**#۴۱ `write!` روی `String` بدون تخصیص اضافه:**
```rust
use std::fmt::Write;
let mut out = String::with_capacity(64);
let _ = write!(out, "{a}-{b}");
```
🎯 `let _ =` لازم است چون `write!` روی String خطا نمی‌دهد ولی `Result` می‌دهد.

**#۴۲ `String::with_capacity` / `Vec::with_capacity`:** جلوگیری از
دوباره‌تخصیص‌های پیاپی در حلقه (به «۱۴ کارایی» نگاه کن).

**#۴۳ `&str` از `String`:** `&s` یا `s.as_str()` — ارزان و خودکار (Deref).

**#۴۴ `char::from_u32` + `decode_utf16`:** کار با یونیکد سطح پایین.

**#۴۵ `normalize`?** — NFC/NFD در std نیست؛ crate `unicode-normalization`.

**#۴۶ `to_lowercase()`/`to_uppercase()` کاراکترمحورند** (نه بایت) — برای
فارسی/ترکی رفتار درست دارد؛ ولی `to_ascii_lowercase()` سریع‌تر و بایت‌محور است.

---

## د) ایتراتور و بستار (#۴۷–#۶۰)

**#۴۷ زنجیره‌های تنبل:** `map`/`filter`/`take`/`skip`/`chain`/`flat_map`/
`scan`/`zip`/`step_by`/`peekable`/`enumerate`. هیچ‌کدام تا `collect`/حلقه اجرا
نمی‌شوند.

**#۴۸ `collect` هوشمند:** نه فقط `Vec`:
```rust
let s: String = chars.collect();
let m: std::collections::HashMap<_,_> = pairs.collect();     // ← از زوج‌ها
let (ok, err): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
```
⚠️ برای `partition` روی `Result` باید هر دو طرف تایپ کامل بدهی.

**#۴۹ `fold` به‌جای حلقه دستی:**
```rust
let total = items.iter().fold(0, |acc, x| acc + x.qty);
```

**#۵۰ `try_fold` / `collect::<Result<Vec<_>,_>>()`:** تجمع با خطا.
```rust
let nums: Result<Vec<i32>, _> = strs.iter().map(|s| s.parse::<i32>()).collect();
```

**#۵۱ `reduce` برای بدون مقدار اولیه.**

**#۵۲ `sum`/`product`/`min`/`max`/`min_by_key`/`count`:** پایان‌دهنده‌های آماده.

**#۵۳ `windows`/`chunks`/`chunks_exact`/`array_chunks`:** دید پنجره‌ای.
```rust
for w in v.windows(3) { /* سه‌تایی‌های لغزان */ }
```

**#۵۴ `sort_by_key` یا `sort_by` با `then_with` برای ترتیب چندسطحی:**
```rust
v.sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight())
                     .then_with(|| b.id.cmp(&a.id)));
```

**#۵۵ `dedup_by_key` و `binary_search_by`** در `Vec` (نوع `Vec`, نه ایتراتور).

**#۵۶ `retain` برای حذف شرطی — بدون حلقه دستی.**
```rust
v.retain(|t| !t.done);
```

**#۵۷ بستار به‌عنوان آرگومان، سه flavor:**
```rust
fn run<F: Fn()>(f: F)                 // می‌تواند چند بار صدا زده شود
fn run<F: FnMut()>(mut f: F)          // حالت تغییر می‌دهد
fn run<F: FnOnce()>(f: F)             // یک بار (ارزش را مصرف می‌کند)
```

**#۵۸ `move` در بستار:** مالکیت را به داخل بستار می‌برد — لازم برای
`thread::spawn` و `'static`.

**#۵۹ `impl Fn` در بازگشتی:** `fn make<'a>(x: &'a i32) -> impl Fn() -> i32 + 'a`.

**#۶۰ `if let Some(x) = f()` + `let-else`:**
```rust
let Some(x) = maybe else { return None; };      // ← تمیزترین راه خروج زودهنگام
```

---

## ه) الگو و کنترل جریان (#۶۱–#۶۸)

**#۶۱ الگوی نابودساز در `match`:**
```rust
match point { Point { x: 0, y } => …, Point { x, .. } => … }
```

**#۶۲ `@` برای نام‌گذاری + تست:**
```rust
match n { x @ 1..=5 => println!("کوچک: {x}"), _ => {} }
```

**#۶۳ `|` در الگو:** `Some(1 | 2 | 3)`.

**#۶۴ `ref`/`ref mut` در الگو** (برای بدهی مالکیت به بیرون):
```rust
let Some(ref mut t) = tasks.get_mut(0) else { return };
```

**#۶۵ `matches!` برای شرط‌های تک‌خطی** (بخش ۱۰).

**#۶۶ حلقه با مقدار:** `loop { break value; }` — `while` و `for` نمی‌توانند
مقدار بدهند.

**#۶۷ برچسب حلقه:**
```rust
'outer: for i in 0..3 { for j in 0..3 { if i + j == 2 { break 'outer; } } }
```

**#۶۸ `if` به‌عنوان عبارت:** `let x = if ok { 1 } else { 2 };` (هر دو شاخه
هم‌نوع).

---

## و) خطا (#۶۹–#۷۶)

**#۶۹ `Option` و `Result` ابزارند، نه تشریفات** — برای «ممکن است نباشد» و
«ممکن است شکست بخورد».

**#۷۰ `?` + `From`:** خطاهای مختلف خودکار به خطای بازگشتی تبدیل می‌شوند.
```rust
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let s = std::fs::read_to_string("f")?;    // io::Error → Box<dyn Error>
    let n: i32 = s.trim().parse()?;           // ParseIntError → Box<dyn Error>
    Ok(())
}
```

**#۷۱ خطای سفارشی:**
```rust
#[derive(Debug)]
enum AppError { Usage(String), NotFound(u64), Io(std::io::Error) }
impl std::fmt::Display for AppError { … }
impl std::error::Error for AppError {}
impl From<std::io::Error> for AppError { … }     // ← `?` کار می‌کند
```

**#۷۲ `map_err` با بستار صریح:** وقتی `Box::from` ابهام‌آور است، نوع را صریح
بنویس:
```rust
.map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
```

**#۷۳ `.unwrap_or_default()` / `.unwrap_or_else(|| …)`** جای `unwrap` در
مسیرهای رایج.

**#۷۴ `expect` با پیام فارسی:** هر `unwrap` را با دلیل گویا عوض کن.

**#۷۵ `std::process::exit`** برای کد بازگشتی معنی‌دار (ما `2` را برای خطای
ورودی می‌دهیم).

**#۷۶ زنجیره‌سازی context (دستی):**
```rust
.map_err(|e| AppError::Usage(format!("خطای خواندن فایل داده: {e}")))?
```
(Crate `anyhow`/`thiserror` این کار را راحت‌تر می‌کند — ۱۶ را ببین.)

---

## ز) ماکرو، متا و صفات (#۷۷–#۸۶)

**#۷۷ `const fn`:** تابعی که در زمان کامپایل هم کار می‌کند (و در ثابت‌ها
استفاده می‌شود). هرچه منطق ساده‌تر، احتمال `const` شدنی بیشتر.

**#۷۸ `const` به‌جای `static`:** ثابت‌ها در محل استفاده درون‌خطی (inline)
می‌شوند؛ `static` آدر‌ دارد.

**#۷۹ `static` برای وضعیت سراسری:** فقط با `AtomicX`/`Mutex`/`OnceLock`.

**#۸۰ `ENV`:** `env!("X")` زمان کامپایل خراب می‌شود اگر نباشد →
`option_env!("X")` یا `std::env::var("X").ok()` زمان اجرا.

**#۸۱ `#[inline]`/`#[cold]`:** فقط بعد از پروفایل. `#[inline]` روی توابع
کوچکِ موازی‌شده مؤثر است.

**#۸۲ `#[track_caller]`:** تا `panic!`/خطا مکان «فراخوان» را نشان دهد نه خود تو.

**#۸۳ `#[must_use]`:** کامپایلر هشدار می‌دهد اگر مقدار برگشتی نادیده بماند.

**#۸۴ `cfg!` در برابر `#[cfg]`:** `#[cfg]` کد را حذف می‌کند؛ `cfg!` فقط
مقدار `bool` می‌دهد (هر دو شاخه کامپایل می‌شوند!).

**#৮۵ feature flag:**
```toml
[features]
default = []
fast = []
```
```rust
#[cfg(feature = "fast")] fn path() { … }
```
⚠️ یک dependency را با feature روشن‌کردن = **انتخاب همه‌گیر در فضای workspace**
(features افزودنی‌اند).

**#۸۶ `build.rs`:** اسکریپت زمان ساخت — تولید کد، خواندن نسخه/تاریخ،
`cargo:rerun-if-changed=…`.
```rust
// build.rs
fn main() { println!("cargo:rerun-if-changed=src/web_index.html"); }
```

---

## ح) کارایی (#۸۷–#۹۳)

**#۸۷ اندازه‌گیری کن، حدس نزن:** `perf`, `cargo bench` (با `criterion`),
`hyperfine`.

**#۸۸ تخصیص را از حلقه بیرون بکش:**
```rust
let mut out = String::with_capacity(n * 16);   // ✅ یک بار
// نه: در هر دور String::new()
```

**#۸۹ `entry` API به‌جای دو جست‌وجو:**
```rust
*counts.entry(key).or_insert(0) += 1;         // یک هش، نه دو
```

**#۹۰ `&str`/`&[T]` در امضاها:** انعطاف + بدون کپی.

**#۹۱ `SmallVec`/`ArrayVec`** (crate) یا `[T; N]` برای مجموعه‌های کوچک بدون
تخصیص heap.

**#۹۲ `release` + LTO + codegen-units=1:**
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"          # باینری کوچک‌تر (بدون unwinding)
strip = true             # حذف نمادها
```

**#۹۳ `#[derive(Copy)]` مراقب بزرگ‌شدن:** کپی‌های ضمنی زیاد = کندی.

---

## ط) ابزار، تست، کارگاه (#۹۴–#۱۰۰)

**#۹۴ `clippy` را جدی بگیر:**
```bash
cargo clippy --all-targets -- -D warnings        # هشدار را خطا کن (در CI)
cargo clippy -- -W clippy::pedantic              # سختگیرانه‌تر
```
(فقط دام‌های واقعی را `#[allow]` کن — با دلیل.)

**#۹۵ `rustfmt` برای یکسان‌سازی:**
```bash
cargo fmt --check && cargo fmt
# rustfmt.toml: max_width=100, use_small_heuristics="Max"
```

**#۹۶ تست‌های ماژولی کنار کد (`#[cfg(test)] mod tests`)** + تست‌های
یکپارچه در `tests/` + doctest در کامنت‌ها (`///`).
```rust
/// ```
/// assert_eq!(ar::table::fit_cell("سلام", 6), "سلام  ");
/// ```
```

**#۹۷ `--test-threads` و تست‌های موازی:** تست‌ها به‌صورت پیش‌فرض موازی‌اند؛
اگر وضعیت سراسری دارند از `serial_test` یا `--test-threads=1` استفاده کن.

**#۹۸ فضای کار (workspace) برای پروژه‌های بزرگ:**
```toml
[workspace]
members = ["ar", "ar-cli", "ar-core"]
[workspace.dependencies]
serde = "1"
```

**#۹۹ `cargo` حرفه‌ای:**
```bash
cargo tree -d                 # نسخه‌های تکراری
cargo tree -e features        # ویژگی‌ها از کجا می‌آیند
cargo metadata --format-version 1 | jq
cargo add/rm/upgrade          # (cargo-edit)
cargo install --list
cargo publish --dry-run
cargo doc --no-deps --open
cargo expand / cargo bloat / cargo udeps
```

**#۱۰۰ نام‌گذاری و نظم پروژه (همان چیزی که این ریپو رعایت می‌کند):**
* نام پوشه/فایل: `snake_case` · نوع/trait: `CamelCase` · تابع/متغیر:
  `snake_case` · ثابت: `SCREAMING_SNAKE_CASE`.
* `err.rs`→`errors.rs`, `utils.rs` ممنوع (به‌جایش ماژول معنادار).
* هر ماژول یک مسئولیت؛ `lib.rs` فقط اعلام و بازصادرات.
* در `lib.rs`: `#![warn(missing_docs)]` + مستند ماژول با `//!`.
* README زنده: هر عددی می‌نویسی، از خروجی واقعی `cargo test` بیاور.
* قبل از کامیت: `cargo fmt && cargo clippy -- -D warnings && cargo test`.

---

> 💡 ۱۲ ترفند کاربردی که در همین پروژه پیاده شدند:
> `KNOWN_SWITCHES` (سوئیچ‌های بی‌مقدار) · `#![allow(dead_code)]` به‌جای
> حذف دموها · `unsigned_abs` به‌جای `abs` · مرز کاراکتر در برش رشتهٔ فارسی ·
> `?Sized` برای تابع روی `dyn` · `#[repr(transparent)]` برای `TaskId` ·
> `include_str!` برای اچ‌تی‌ام‌ال · `BufReader<&TcpStream>` · «Content-Length =
> بایت» · بازیابی از مسمومیت قفل · `test_cases!` برای تولید تست · پیام
> خطای فارسی + پیشنهاد Levenshtein.

[⬅️ قبلی: unsafe و FFI](11-unsafe-and-ffi.md) · [بعدی: دام‌ها ➡️](13-pitfalls.md)
