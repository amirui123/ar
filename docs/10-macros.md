# 🧬 ۱۰ — ماکروها

ماکرو = کد که کد می‌سازد. در Rust دو خانواده داریم:

| | `macro_rules!` (اعلانی) | ماکروهای رویه‌ای (procedural) |
|---|---|---|
| ورودی | توکن | `TokenStream` |
| نیاز به crate جدا | ❌ | ✅ (دارای `proc-macro = true`) |
| مقایسه | «الگو ← جایگزین» | کد Rust که توکن می‌سازد |
| debuggable | ✅ (`cargo expand`) | ⚠️ سخت‌تر |
| انواع | `macro_rules!` | derive / attribute / function-like |

---

## ۱) `macro_rules!` — پایه‌ها

```rust
macro_rules! two {
    () => { 2 };                     // قانون بدون آرگومان
    ($x:expr) => { $x * 2 };         // یک expr
}

println!("{} {}", two!(), two!(21)); // 2 42
```

### انواع «فرگمنت» (fragment specifier)

| نشانه | می‌گیرد | نمونه |
|-------|---------|-------|
| `$x:expr` | یک عبارت | `1 + 2` |
| `$x:ident` | نام | `foo`, `عبارت` |
| `$x:ty` | نوع | `Vec<u8>` |
| `$x:pat` | الگو | `Some(x)` |
| `$x:stmt` | دستور | `let a = 1;` |
| `$x:block` | بلوک | `{ … }` |
| `$x:item` | آیتم | `fn f() {}` |
| `$x:literal` | لیترال | `42`, `"a"` |
| `$x:tt` | یک توکن درخت | هر چیزی |
| `$x:path` | مسیر | `std::vec::Vec` |
| `$x:meta` | محتوای اتریبیوت | `cfg(test)` |
| `$x:vis` | سطح دسترسی | `pub(crate)` |
| `$x:lifetime` | طول عمر | `'a` |

🎯 **قاعدهٔ طلایی:** تا وقتی ممکن است از `expr`/`ident`/`ty` استفاده کن، نه
`tt` — چون `tt` خطاهای بدتری می‌دهد و بعداً امکان گسترش ندارد.

### تکرار `$( … ),*`
```rust
macro_rules! sum {
    ($($x:expr),* $(,)?) => {{          // $(,)? = ویرگول انتهایی اختیاری
        let mut total = 0;
        $( total += $x; )*               // ← همان تکرار روی بدنه
        total
    }};
}
let t = sum!(1, 2, 3,);
```
تکرارها: `$( … )*` (صفر یا بیشتر) · `$( … )+` (یک یا بیشتر) ·
`$( … )?` (صفر یا یک) · جداکننده‌ها: `,` `;` یا هیچ.

### الگوهای چند قانونی و ترتیب
```rust
macro_rules! my_vec {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => {{
        let mut v = Vec::new();
        $( v.push($x); )+
        v
    }};
}
```
📌 **ترتیب مهم است:** ماکرو اولین قانونی را که تطبیق کند اجرا می‌کند → قانون
عمومی را **آخر** بنویس.

---

## ۲) ترفندهای `macro_rules!`

```rust
// ۱) hygiene: متغیرهای داخل ماکرو با بیرون تداخل نمی‌کنند
macro_rules! make_var { () => { let x = 1; } }
macro_rules! shadow { ($val:expr) => {{ let x = $val; x * 2 }} }
let x = 10;
let y = shadow!(5);      // x داخلی با x بیرونی فرق دارد ✅

// ۲) «sorting» الگوها برای انتخاب نوع
macro_rules! kind {
    (str)     => { "رشته" };
    (num)     => { "عدد" };
    ($other:tt) => { "دیگر" };
}

// ۳) ماکرو برای کاهش تکرار تست (همان الگوی macros_demo.rs)
macro_rules! test_cases {
    ($($label:ident: $input:expr => $want:expr;)+) => {
        $(
            #[test]
            fn $label() { assert_eq!(f($input), $want); }   // ← نام از $label
        )+
    };
}
test_cases! {
    empty_ok: 0 => 0;
    one_ok:   1 => 1;
}

// ۴) بازگشت به خود (recursion) برای ساختن JSON/HTML
macro_rules! json_like {
    ([ $($v:tt),* ]) => { format!("[{}]", vec![$(json_like!($v)),*].join(",")) };
    (null) => { "null".to_string() };
    ($v:literal) => { $v.to_string() };
}

// ۵) تشخیص نوع با trick تخصصی‌سازی (autoref specialization)
//    الگوی libهای معروف (anyhow, serde) — ماکروی «اگر T: Display بود این، وگرنه آن».

// ۶) ماکرو برای تولید ساختار با پیشوند
macro_rules! ids {
    ($($name:ident),*) => { $( pub struct $name(pub u64); )* };
}
ids!(UserId, PostId);

// ۷) کامپایل‌تایم fn که خطای خوانا بدهد
macro_rules! require_odd {
    ($n:literal) => {
        const _: () = assert!($n % 2 == 1, "عدد باید فرد باشد!");
    };
}
require_odd!(3);     // ✅
// require_odd!(4);  // ❌ خطای کامپایل: عدد باید فرد باشد!
```

---

## ۳) اتریبیوت‌های داخلی مهم

```rust
#[macro_export]                 // ← از بیرون crate قابل استفاده
#[doc(hidden)]                  // ← پنهان از مستندات (ماکروهای داخلی)
#[allow(dead_code)]
#[cfg(test)]
#[cfg(feature = "x")]
#[cfg_attr(test, derive(Debug))]
#[deprecated(since = "1.0", note = "از new استفاده کن")]
#[inline] / #[inline(always)] / #[cold] / #[must_use] / #[track_caller]
#[repr(C)] / #[repr(transparent)] / #[repr(u8)]
#[non_exhaustive]               // ← users نمیتوانند exhaustive match کنند
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
```

---

## ۴) ماکروهای استاندارد که باید بشناسی

```rust
vec![1, 2, 3]                  // ساخت Vec
format!("{x}")                 // رشتهسازی
println!/print!/eprint/write!  // خروجی
concat!("a", "b")              // ← در زمان کامپایل!
stringify!(1 + 1)              // ← "1 + 1"
include_str!("file.txt")       // ← محتوای فایل در باینری (کاری که web.rs می‌کند)
include_bytes!(..) / include!(..)
env!("HOME") / option_env!("X")
cfg!(debug_assertions)         // → bool قابل استفاده در کد
assert!/assert_eq!/assert_ne!  // + پیام سفارشی
debug_assert!/debug_assert_eq!
matches!(x, Some(_))           // ← تمیزترین راه تست الگو
todo!() / unimplemented!() / unreachable!()
dbg!(x)                        // ← چاپ + مقدار + فایل/خط (فقط دیباگ)
file!() / line!() / column!() / module_path!()
thread_local! {} / lazy_static!-like
try! / ?                       // try! منسوخ شد
select! (crate) / join! (crate)
paste::paste! (crate)          // چسباندن نامها
```

🎯 `matches!` ترفند کوچک اما پرکاربرد:
```rust
if matches!(cmd.as_str(), "add" | "rm" | "done") { }
```

---

## ۵) ماکروهای رویه‌ای (procedural)

نیازمند crate جدا با `proc-macro = true` و crateهای `syn`/`quote`:

```rust
// lib.rs در crate جدا
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    quote! {
        impl Describe for #name {
            fn describe(&self) -> String { stringify!(#name).to_string() }
        }
    }.into()
}
```

| نوع | صفت | نمونه |
|-----|-----|-------|
| derive | `#[proc_macro_derive(X)]` | `#[derive(Serialize)]` |
| attribute | `#[proc_macro_attribute]` | `#[tokio::main]`, `#[test]` |
| function-like | `#[proc_macro]` | `sql!(...)`, `include!(...)` |

📌 **ترفند:** ماکروی رویه‌ای نمی‌تواند «نام» را ببیند اگر از `syn` ساختاری
استفاده نکنی؛ `syn::DeriveInput` همه چیز را می‌دهد. برای کار ساده، `quote!`
کافی است و ترتیب `#var` = درج متغیر.

### کامپایل‌کردن ماکروها
```bash
cargo install cargo-expand
cargo expand            # ← کد واقعی بعد از همهٔ ماکروها (بهترین ابزار یادگیری!)
cargo expand --lib macros_demo
```

---

## ۶) مغز متفکر: کِی ماکرو بنویسیم؟

| ✅ خوب | ❌ بد |
|-------|------|
| حذف تکرار تکراری (boilerplate) | جایگزین تابع |
| API نوع‌محور (مثل `vec!`) | قواعد پیچیده که کسی نمی‌فهمد |
| تولید تست‌های زیاد | پنهان‌کردن منطق تجاری |
| DSL واقعی (SQL، HTML) | جای trait |
| کدی که باید «در زمان کامپایل» اجرا شود | کد یک‌بارمصرف |

⚠️ **هزینهٔ ماکرو:** پیام خطای بدتر، زمان کامپایل بیشتر، دیباگ سخت‌تر، ابزار
(IDE) ضعیف‌تر. اگر تابع یا جنریک کار می‌کند، همان را بزن.

---

## ۷) ترفندهای سطح بالا

```rust
// ۱) ماکرو با «آرگومان کلیدواژه‌ای» (شبیه named args)
macro_rules! config {
    ($($k:ident : $v:expr),* $(,)?) => {
        vec![$( (stringify!($k), format!("{}", $v)) ),*]
    };
}
let cfg = config!(port: 8080, debug: true);

// ۲) spread کردن پیاده‌سازی روی انواع (impl برای چند نوع)
macro_rules! impl_id {
    ($($t:ty),*) => { $( impl $t { fn kind(&self) -> &'static str { stringify!($t) } } )* };
}
impl_id!(u8, u16, u32, u64);

// ۳) ماکرو که تعداد آرگومان‌ها را می‌شمارد (tt-munching)
macro_rules! count {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}
const N: usize = count!(a b c d e);

// ۴) افزودن #[test] برای هر فایل داده
// (به جای نوشتن ۲۰ تست دستی)

// ۵) ماکرو برای time! و اندازه‌گیری
macro_rules! time_it {
    ($label:literal, $e:expr) => {{
        let __t = std::time::Instant::now();
        let __v = $e;
        println!("⏱ {} → {:?}", $label, __t.elapsed());
        __v
    }};
}
```

---

## ۸) debug و بررسی ماکرو

```bash
cargo expand my_crate::demos::macros_demo   # کد تولیدشده
rustc -Zunpretty=expanded file.rs           # (nightly)
cargo build 2>&1 | head -50                 # خطاها اغلب «در ماکرو» را نشان می‌دهند
```
📌 `#[macro_export]` ماکرو را در `crate::` می‌گذارد، **نه** در ماژول فعلی
(دام قدیمی؛ الان `pub use` هم رفته).

[⬅️ قبلی: async](09-async.md) · [بعدی: unsafe و FFI ➡️](11-unsafe-and-ffi.md)
