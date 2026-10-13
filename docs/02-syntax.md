# 📝 ۰۲ — سینتکس ریزبه‌ریز Rust

> این سند یک «مرجع فشرده» است: هر ساختار نحوی + معنی‌اش + ترفندش.

## ۱) متغیرها

```rust
let x = 5;              // تغییرناپذیر (immutable) — پیش‌فرض!
let mut y = 5;          // تغییرپذیر
y += 1;

const MAX: u32 = 100;   // ثابتِ زمان کامپایل (نیازمند نوع، همیشه بی‌تغییر)
static GREETING: &str = "سلام";   // جای ثابت در حافظه، آدرس‌دار، 'static

let (a, b) = (1, 2);    // تجزیهٔ tuple
let [c, d] = [3, 4];    // تجزیهٔ آرایه
let _ = compute();      // نادیده گرفتن نتیجه (و رهاکردن فوری مقدار)
let _x = compute();     // مقدار تا انتهای بلوک زنده می‌ماند ← تفاوت مهم!
```

**قاعدهٔ طلایی:** تا وقتی مجبور نشدی `mut` نگذار. کامپایلر خودش خطا می‌دهد
اگر لازم شد — و این «حداقل تغییرپذیری» باعث می‌شود کد را راحت‌تر بفهمی.

### سایه‌انداختن (shadowing)
```rust
let spaces = "   ";        // &str
let spaces = spaces.len();  // usize — همان نام، نوع جدید ✅
```
متفاوت از `mut`: نوع عوض می‌شود و مقدار قدیمی «مصرف» می‌شود.

---

## ۲) انواع داده

### عددی
| نوع | اندازه | محدوده |
|-----|--------|--------|
| `i8/u8` | ۱ بایت | ‎-128..127 / 0..255 |
| `i16/u16` | ۲ بایت | ±32 هزار |
| `i32/u32` | ۴ بایت | ±2.1 میلیارد (**پیش‌فرض عدد صحیح**) |
| `i64/u64` | ۸ بایت | ±9.2 × 10¹⁸ |
| `i128/u128` | ۱۶ بایت | خیلی بزرگ |
| `isize/usize` | اندازهٔ معماری | برای ایندکس و طول |
| `f32/f64` | ۴/۸ بایت | اعشاری (**f64 پیش‌فرض**) |

```rust
let a = 42;          // i32
let b = 42u8;        // با پسوند
let c = 1_000_000;   // جداکنندهٔ بصری
let d = 0b1010;      // دودویی
let e = 0o17;        // هشت‌هشتی
let f = 0xFF_FF;     // شانزده‌شانزدهی
let g = b'A';        // u8 (بایت)
let h = 'A';         // char (۴ بایت، یونیکد)
let i = 1.5e3;       // ۱۵۰۰.۰
```

### رشته و کاراکتر
```rust
let s1: &str = "ثابت";         // برش رشته (اشاره‌گر + طول)
let s2: String = s1.to_string();
let c: char = 'س';             // نکته: ۴ بایت، نه ۱!
let bytes: &[u8] = s1.as_bytes();
```
> 💣 **مهم‌ترین دام:** `s.len()` بایت است، نه کاراکتر. `"سلام".len() == 8` ولی
> `"سلام".chars().count() == 4`. و `&s[0..1]` **پنیک می‌کند** چون وسط کاراکتر است.

### مجموعه‌ها
```rust
let t: (i32, &str, f64) = (1, "دو", 3.0);   // tuple — طول ثابت
let arr: [i32; 3] = [1, 2, 3];              // آرایه روی پشته، طول بخشی از نوع
let arr2 = [0u8; 1024];                     // آرایهٔ ۱۰۲۴ بایتی پر از صفر
let v: Vec<i32> = vec![1, 2, 3];            // بردار روی heap، قابل رشد
let s: &[i32] = &v;                         // برش (اشاره‌گر + طول)
```
| | طول | حافظه | قابل رشد |
|---|---|---|---|
| `[T; N]` | ثابت در زمان کامپایل | پشته | ❌ |
| `Vec<T>` | پویا | heap | ✅ |
| `&[T]` | پویا | قرض از دیگری | ❌ |
| `&str` | پویا | قرض | ❌ |
| `String` | پویا | heap | ✅ |

### Option و Result
```rust
let a: Option<i32> = Some(5);
let b: Option<i32> = None;
let r: Result<i32, String> = Ok(1);
```
`Option` و `Result` **enum** هستند (نه استثنا) — همین باعث می‌شود کامپایلر
مجبورت کند حالت «نیست» را رسیدگی کنی.

### انواعی که خودت می‌سازی
```rust
struct Point { x: f64, y: f64 }                  // ساختار نام‌دار
struct Meters(f64);                              // tuple struct (newtype)
struct Unit;                                     // بدون فیلد (نشانگر)

enum Shape {                                     // enum با داده
    Circle { r: f64 },
    Rect(f64, f64),
    Point,
}
```

---

## ۳) توابع

```rust
fn add(a: i32, b: i32) -> i32 { a + b }   // ⚠️ بدون `;` = «مقدار بازگشتی»

fn early(x: i32) -> i32 {
    if x < 0 { return 0; }   // return زودهنگام (با ;)
    x * 2                    // آخرین عبارت = مقدار
}

fn nothing() {}              // بدون بازگشتی = ()

fn generic<T: Clone>(x: T) -> T { x.clone() }
fn impl_arg(x: impl std::fmt::Display) { println!("{x}"); }
fn multiple(a: i32, b: i32) -> (i32, i32) { (a + b, a - b) }  // چند خروجی با tuple
```

### اشاره‌گر تابع در برابر بستار
```rust
fn double(x: i32) -> i32 { x * 2 }
let f: fn(i32) -> i32 = double;         // اشاره‌گر تابع (بدون محیط)
let c = |x| x * 2;                       // بستار (می‌تواند محیط بگیرد)
let g: &dyn Fn(i32) -> i32 = &double;    // پویا
let h: Box<dyn Fn(i32) -> i32> = Box::new(c);
```

### `#[inline]`, `#[cold]`, `#[must_use]`
```rust
#[inline]                       // درخواست درون‌ریزی (compler خودش هم بلد است)
pub fn tiny(&self) -> u32 { self.0 }

#[must_use]                     // اگر نتیجه را نادیده بگیری، هشدار می‌گیری
pub fn compute() -> u32 { 42 }

#[must_use = "خطا را بررسی کن!"]   // پیام هشدار دلخواه
pub fn parse(s: &str) -> Result<u32, String> { /* … */ }
```

---

## ۴) کنترل جریان

```rust
if x > 0 { } else if x == 0 { } else { }

// if یک *عبارت* است → می‌شود مقدار داد:
let label = if x > 0 { "مثبت" } else { "منفی" };   // هر دو شاخه یک نوع!

match value {
    0 => "صفر",
    1 | 2 => "کم",                 // الگوی «یا»
    3..=9 => "میانی",              // بازه
    n if n % 2 == 0 => "زوج",      // نگهبان (guard)
    n @ 10..=99 => "دورقمی",       // @ = نام‌گذاری + شرط
    _ => "بزرگ",                   // _ = هر چیز دیگر (جامعیت)
}

while cond { }
while let Some(x) = iter.next() { }   // تا وقتی الگو بخواند
loop { break 42; }                     // حلقهٔ بی‌پایان با مقدار خروجی
for i in 0..10 { }                     // 0..10 = 0 تا 9 (بدون ۱۰!)
for x in &v { }                        // قرض (v دست‌نخورده می‌ماند)
for (i, x) in v.iter().enumerate() { }
'outer: for i in 0..3 {                // برچسب حلقه (label)
    for j in 0..3 {
        if j == 1 { continue 'outer; }
        if i == 2 { break 'outer; }
    }
}
```

### `?` — اپراتور انتشار خطا
```rust
fn read() -> std::io::Result<String> {
    let s = std::fs::read_to_string("f.txt")?;  // خطا؟ همان‌جا برگردان
    Ok(s.to_uppercase())
}
// معادل:
fn read_long() -> std::io::Result<String> {
    match std::fs::read_to_string("f.txt") {
        Ok(s) => Ok(s.to_uppercase()),
        Err(e) => return Err(e),
    }
}
```

---

## ۵) الگوها (جایی که Rust واقعاً می‌درخشد)

```rust
// ۱) تفکیک enum
match msg {
    Msg::Quit => {},
    Msg::Move { x, y } => {},          // ساختار نام‌دار
    Msg::Write(text) => {},            // tuple
    Msg::ChangeColor(r, g, b) => {},
}

// ۲) تفکیک tuple و ساختار
let Point { x, y } = p;
let (a, .., z) = (1, 2, 3, 4);        // .. = نادیده بگیر وسط را

// ۳) الگوهای برش
match slice {
    [] => "خالی",
    [only] => "یکی",
    [first, .., last] => "چندتا",
}

// ۴) مراجع و ref
let s = String::from("x");
match &s { Some(_) => {}, None => {} }     // الگو روی قرض
let Some(v) = maybe else { return };       // let-else (زودهنگام برگردان)

// ۵) در پارامتر تابع!
fn dist((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> f64 { /* … */ }
```

---

## ۶) ماژول‌ها و مسیرها

```rust
mod inner {                     // ماژول درون‌خطی
    pub fn f() {}
    pub(crate) fn only_crate() {}     // فقط داخل همین کریت
    pub(super) fn only_parent() {}    // فقط والد
    pub(in crate::inner) fn scoped() {}
    mod deeper;                 // از فایل src/inner/deeper.rs می‌خواند
}

use std::collections::HashMap;              // آوردن نام به دامنه
use std::collections::{HashMap, HashSet};   // چندتا با هم
use std::io::Write as _;                    // ✅ استفاده از صفت بدون نام
use crate::model::Task;                     // مسیر مطلق داخل کریت
use super::errors::AppError;                // والد
use self::inner::f;                         // همین ماژول

pub mod api;                                // ماژول عمومی (برای بیرون)
```

### نکات واقعی
* 🎯 `use std::io::Write as _;` = صفت (trait) لازم است ولی *نامش* را نمی‌خواهی
  → نه هشدار «import استفاده نشد»، نه شلوغی نام. (در `store.rs` ما همین را داریم.)
* 🎯 اگر `use` فقط در تستها لازم است، در `mod tests` بگذارش.
* 🎯 `crate::` = ریشهٔ همین کریت. `::` در ابتدا = کریت بیرونی (سبک قدیمی، حالا `::std` هم معمول است).

---

## ۷) اپراتورها

| دسته | عملگرها |
|------|---------|
| حساب | `+ - * / %` , `+= -= *= /= %=` |
| مقایسه | `== != < > <= >=` |
| منطقی | `&&` (اتصال کوتاه)، `\|\|`، `!` |
| بیت | `& \| ^ ! << >>` , `&= \|= ^= <<= >>=` |
| دامنه | `..` (بدون پایان)، `..=` (شامل)، `a..b` |
| ارجاع | `&`, `&mut`, `*` (deref) |
| دسترسی | `.` (متد/فیلد)، `::` (مسیر/مقدار وابسته)، `?` (انتشار خطا) |
| تایپ | `as` (تبدیل کور)، `_` (استنتاج) |

⚠️ `..` و `..=` را قاطی نکن:
```rust
for i in 0..3 { }    // 0, 1, 2
for i in 0..=3 { }   // 0, 1, 2, 3
let seg = &v[1..3];  // ایندکس ۱ و ۲ (بدون ۳)
```

---

## ۸) ویژگی‌های (attributes) پرکاربرد

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]  // تولید کد
#[allow(dead_code)] #[warn(unused)] #[deny(unsafe_code)] #[forbid(...)] // کنترل لینت
#[inline(always)] #[cold] #[track_caller]
#[cfg(test)] #[cfg(feature = "json")] #[cfg(target_os = "linux")]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[deprecated(since = "0.2.0", note = "از `new_v2` استفاده کن")]
#[repr(C)] #[repr(u8)] #[repr(transparent)] #[repr(packed)]
#[non_exhaustive]     // کاربران باید `_ =>` بگذارند (سازگاری نسخه‌های بعدی)
#[must_use]
#[doc = "…"]  و  /// مستندات
#[doc(hidden)]        // از مستندات عمومی پنهان کن (ولی در دسترس)
#[test] #[bench] #[ignore]
```

---

## ۹) چیزهایی که تازه‌کارها اشتباه می‌کنند

| می‌نویسی | درست |
|---------|------|
| `let x = 5; x = 6;` | `let mut x = 5;` |
| `if (x > 0) { }` | `if x > 0 { }` (پرانتز لازم نیست، هشدار می‌دهد) |
| `fn f() -> i32 { return x; }` | `fn f() -> i32 { x }` |
| `let s = String::new("x")` | `let s = String::from("x")` یا `"x".to_string()` |
| `arr[10]` با احتمال بیرون‌زدن | `arr.get(10)` که `Option` می‌دهد |
| `for i in 0..v.len() { v[i] }` | `for x in &v { }` (امن‌تر و سریع‌تر) |
| `if x == true` | `if x` |
| `if x != None` | `if x.is_some()` |
| `s.len() == 0` | `s.is_empty()` |
| `while i < n { …; i += 1 }` | `for i in 0..n { }` |
| `.unwrap()` همه‌جا | `?` یا `unwrap_or` / `expect("دلیل")` |

---

## ۱۰) تفاوت‌هایی که از زبان‌های دیگر عجیب به‌نظر می‌آیند

| در زبان‌های دیگر | در Rust |
|------------------|---------|
| `null` | `Option<T>` — کامپایلر مجبورت می‌کند چک کنی |
| استثنا (try/catch) | `Result<T, E>` + `?` — خطا در امضای تابع دیده می‌شود |
| `class` با وراثت | `struct` + `trait` (ترکیب به‌جای وراثت) |
| Garbage Collector | مالکیت + عمر (هزینهٔ صفر در زمان اجرا) |
| بازگشت چند مقدار با آرایه | `tuple` یا `struct` |
| `switch` با `break` | `match` که **مقدار** می‌دهد و جامعیت دارد |
| `for (i=0; …)` | `for x in iter` — حلقه روی *مقدار*، نه ایندکس |
| Overloading تابع | ژنریک + `trait` |
| وراثت enum بدون داده | `enum` با داده (Algebraic Data Type) |

[⬅️ قبلی: نصب و cargo](01-install-and-cargo.md) · [بعدی: مالکیت ➡️](03-ownership.md)
