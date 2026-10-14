//! # دموی ماکروها — کدی که کد می‌نویسد
//!
//! در Rust دو نوع ماکرو داریم:
//! 1. **اعلانی** (`macro_rules!`) — الگو-محور، همان چیزی که اینجا یاد می‌گیریم.
//!    مثل `println!`، `vec!`، `matches!`.
//! 2. **رویه‌ای** (procedural) — با کد Rust روی درخت نحو کار می‌کنند؛
//!    نیاز به crate جدا دارند (`#[derive(Serialize)]` از این نوع است).
//!
//! ⚠️ **اول تابع، بعد ماکرو.** ماکرو فقط وقتی لازم است که:
//! * تابع نمی‌تواند (تعداد آرگومان متغیر، یا `return` از تابع فراخوان)
//! * می‌خواهی کد *تولید* کنی (پیاده‌سازی صفت برای چند نوع)
//! * می‌خواهی اطلاعات زمان کامپایل بگیری (`file!()`, `line!()`)


#![allow(dead_code)] // در فایل‌های دمو بعضی آیتم‌ها فقط «نمایشی»‌اند و همه‌جا مصرف نمی‌شوند
use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) اولین ماکرو و چند شاخه (arm)"));
    basic();

    println!("\n{}", color::bold("۲) تکرار (repetition) — قدرت اصلی"));
    repetition();

    println!("\n{}", color::bold("۳) «جویدن TT» — ماکروی پردازش مرحله‌ای"));
    tt_munching();

    println!("\n{}", color::bold("۴) اطلاعات زمان کامپایل"));
    compile_time_info();

    println!("\n{}", color::bold("۵) ماکرو برای تولید تست"));
    test_generation();

    println!("\n{}", color::bold("۶) تأیید در زمان کامپایل"));
    compile_time_checks();
}

// ═══════════════════════════════════════════════════════════════════════════
/// سلام ساده — سه شاخه برای سه شکل فراخوانی.
///
/// 🔑 `$name:ident` یک «قطعه» (fragment specifier) است؛ انواع مهمش:
/// `expr` (عبارت)، `ident` (نام)، `ty` (نوع)، `literal`، `pat` (الگو)،
/// `path`، `block`، `stmt`، `tt` (درخت توکن)، `item`.
macro_rules! greet {
    // شاخه ۱: بدون آرگومان
    () => {
        println!("    سلام!")
    };
    // شاخه ۲: با نام
    ($name:expr) => {
        println!("    سلام {}!", $name)
    };
    // شاخه ۳: با نام و عنوان (ترتیب اهمیت دارد: خاص‌ترین اول)
    ($name:expr, $title:expr) => {
        println!("    سلام {} {}!", $title, $name)
    };
}

/// ساخت آدرس وب.
macro_rules! url {
    ($host:expr) => {
        format!("https://{}", $host)
    };
    ($host:expr, $path:expr) => {
        format!("https://{}/{}", $host, $path.trim_start_matches('/'))
    };
    ($host:expr, $($segment:expr),+) => {{
        // تکرار: هر قطعه را با «/» به هم بچسبان
        let parts: Vec<String> = vec![$($segment.to_string()),+];
        format!("https://{}/{}", $host, parts.join("/"))
    }};
}

fn basic() {
    greet!();
    greet!("دنیا");
    greet!("علی", "آقای");

    println!("  url!({}) = {}", "\"example.com\"", url!("example.com"));
    println!("  url! با مسیر = {}", url!("example.com", "/api/tasks"));
    println!("  url! با چند قطعه = {}", url!("example.com", "api", "v2", "tasks"));

    // 🌟 نکتهٔ «بهداشت» (hygiene): متغیرهای داخل ماکرو با محیط بیرون قاطی نمی‌شوند.
    let x = 1;
    macro_rules! make_x {
        () => {{
            let x = 999; // این x با بیرونی کاری ندارد
            x
        }};
    }
    println!("  hygiene: بیرونی x={x}، درونی x={}", make_x!());
}

// ═══════════════════════════════════════════════════════════════════════════
/// `vec!` خودمان.
macro_rules! my_vec {
    // حالت خالی
    () => { Vec::new() };
    // حالت تکرار: `my_vec![0; 5]`
    ($elem:expr; $n:expr) => {{
        // `vec![x; n]` داخل کتابخانهٔ خودمان (std) را صدا می‌زنیم
        std::vec::from_elem($elem, $n)
    }};
    // حالت فهرست: با کاما اختیاری در آخر
    ($($x:expr),+ $(,)?) => {{
        // 🔑 الگوی کلاسیک: تعریف متغیر محلی + push
        let mut v = Vec::new();
        $( v.push($x); )+
        v
    }};
}

/// «نگاشت» روی چند مقدار — چند ورودی، چند خروجی.
macro_rules! collect_all {
    ($($e:expr),* $(,)?) => {{
        let mut out = Vec::new();
        $( out.push($e.to_string()); )*
        out
    }};
}

/// ماکروی لاگ با سطح — `log!(warn, "…")`
macro_rules! log_msg {
    (info, $($arg:tt)*) => { println!("    ℹ️  {}", format!($($arg)*)) };
    (warn, $($arg:tt)*) => { println!("    ⚠️  {}", format!($($arg)*)) };
    (error, $($arg:tt)*) => { println!("    ❌ {}", format!($($arg)*)) };
    // شاخهٔ پیش‌فرض: هر چیز دیگری → info
    ($($arg:tt)*) => { log_msg!(info, $($arg)*) };
}

fn repetition() {
    let v = my_vec![1, 2, 3];
    let z = my_vec![0u8; 4];
    let e: Vec<i32> = my_vec![];
    let trailing = my_vec![1, 2, 3,]; // کاما در آخر هم مجاز

    println!("  my_vec![1,2,3] = {v:?}");
    println!("  my_vec![0u8;4] = {z:?}");
    println!("  my_vec![]      = {e:?}");
    println!("  با کامای آخر   = {trailing:?}");

    println!("  collect_all! = {:?}", collect_all![1, "دو", 3.5, true]);

    log_msg!("پیام پیش‌فرض");
    log_msg!(warn, "مقدار {} عجیب است", 42);
    log_msg!(error, "کد {} رخ داد", 500);
}

// ═══════════════════════════════════════════════════════════════════════════
/// «جویدن TT» (TT munching): ماکروی جمع که هر بار یک عنصر «می‌جود»
/// و برای بقیه بازگشتی خودش را صدا می‌زند.
///
/// این تکنیک برای پردازش‌های پیچیدهٔ نحوی لازم است (وقتی `$(),*` کافی نیست).
macro_rules! sum_all {
    // حالت پایه: فقط یک accumulator مانده
    ($acc:expr) => { $acc };
    // حالت بازگشتی: یک عنصر + بقیه
    ($acc:expr, $next:expr $(, $rest:expr)*) => {
        sum_all!($acc + $next $(, $rest)*)
    };
}

/// ماکروی «ساخت تابع گتر» — کدی که کد می‌سازد.
macro_rules! make_getter {
    ($name:ident, $field:ident, $ty:ty) => {
        /// گتر تولیدشده توسط ماکرو
        pub fn $name(&self) -> $ty {
            self.$field
        }
    };
}

/// ساختار نمونه برای ماکروی بالا.
#[derive(Debug, Default)]
pub struct Metrics {
    /// تعداد درخواست‌ها
    pub requests: u64,
    /// خطاها
    pub errors: u32,
}

impl Metrics {
    make_getter!(requests, requests, u64);
    make_getter!(errors, errors, u32);
}

fn tt_munching() {
    println!("  sum_all!(1, 2, 3, 4, 5) = {}", sum_all!(1, 2, 3, 4, 5));
    println!("  sum_all!(10) = {}", sum_all!(10));

    let m = Metrics { requests: 120, errors: 3 };
    println!("  گترهای تولیدشده: requests()={} errors()={}", m.requests(), m.errors());
    println!("  (توجه: `make_getter!` برای هر فیلد، یک تابع *در زمان کامپایل* نوشت)");
}

// ═══════════════════════════════════════════════════════════════════════════
fn compile_time_info() {
    // این ماکروها مقدار را *در زمان کامپایل* جای‌گذاری می‌کنند (هزینهٔ اجرا: صفر)
    println!("  فایل: {}", file!());
    println!("  خط: {}", line!());
    println!("  ستون: {}", column!());
    println!("  نام ماژول: {}", module_path!());
    println!("  نسخهٔ بسته: {}", env!("CARGO_PKG_VERSION"));
    println!("  نمایهٔ ساخت: {}", if cfg!(debug_assertions) { "debug" } else { "release" });

    // `stringify!` — عبارت را به متن تبدیل می‌کند
    let code = stringify!(1 + 2 * 3);
    println!("  stringify!(1 + 2 * 3) = «{code}»");

    // `concat!` — چسباندن در زمان کامپایل
    const VERSION: &str = concat!("v", "1", ".", "0");
    println!("  concat! = {VERSION}");

    // `include_str!` — خواندن فایل در زمان کامپایل (مثل قالب HTML سرور ما!)
    let readme = include_str!("../../Cargo.toml");
    println!("  include_str! اولین خط Cargo.toml: {}", readme.lines().next().unwrap_or(""));
    println!("  (فایل در باینری حک می‌شود → در زمان اجرا فایلی لازم نیست)");

    // `cfg!` و `#[cfg]` — کامپایل شرطی بر اساس سیستم‌عامل/معماری
    println!("  سیستم: {} / معماری: {}", std::env::consts::OS, std::env::consts::ARCH);

    // 🐞 `dbg!` — ابزار اشکال‌زدایی سریع: هم عبارت، هم فایل/خط، هم مقدار
    let value = dbg!(2 + 3);

    // اینجا برای تمیزی خروجی، جواب را هم چاپ می‌کنیم
    println!("  dbg! مقدار را برگرداند: {value}");
    println!("  (در کد تولیدی dbg! را پاک کن — به stderr می‌نویسد و دیده می‌شود)");
}

// ═══════════════════════════════════════════════════════════════════════════
/// ماکرویی که چند تست تولید می‌کند — ترفندی برای حذف کد تکراری.
macro_rules! test_cases {
    // هر برچسب تبدیل به *نام یک تابع تست مستقل* می‌شود.
    // (نسخهٔ اول این ماکرو همه را با یک نام می‌ساخت و خطای «تعریف تکراری» می‌داد!)
    ($($label:ident => $input:expr, $expected:expr;)*) => {
        $(
            #[test]
            fn $label() {
                let got = $input;
                assert_eq!(got, $expected, "حالت {}", stringify!($label));
            }
        )*
    };
}

test_cases! {
    double_zero => 0 * 2, 0;
    double_two => 2 * 2, 4;
    double_neg => (-3) * 2, -6;
}

fn test_generation() {
    println!("  چهار تست با یک ماکرو ساخته شد (در `cargo test` اجرا می‌شوند)");
    println!("  ترفند: برای تست‌های «جدولی» (table-driven) بی‌نظیر است");
}

// ═══════════════════════════════════════════════════════════════════════════
/// صفت (trait) نمونه برای تأیید در زمان کامپایل.
trait NotSend {
    /// فقط برای نمایش
    fn marker(&self) -> &'static str;
}

impl NotSend for u8 {
    fn marker(&self) -> &'static str {
        "u8"
    }
}

fn compile_time_checks() {
    // ۱) `const { assert!(...) }` — اگر برقرار نباشد، کامپایل شکست می‌خورد.
    const _: () = assert!(std::mem::size_of::<u64>() == 8, "فرض اندازهٔ u64 نقض شد");
    // (این خط فقط کامپایل می‌شود اگر u64 هشت‌بایتی باشد — تضمین معماری)

    // ۲) تأیید «پیاده‌سازی صفت» در زمان کامپایل (الگوی static-assertions)
    fn assert_implements<T: NotSend>() {}
    assert_implements::<u8>(); // اگر u8 صفت را نداشت، کامپایل نمی‌شد
    println!("  u8 صفت NotSend را دارد (در زمان کامپایل تأیید شد)");

    // ۳) تأیید صفت‌های خودکار: Send/Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<String>();
    assert_sync::<String>();
    assert_send::<std::sync::Arc<Vec<u8>>>();
    println!("  String: Send ✓ Sync ✓   Arc<Vec<u8>>: Send ✓");

    // ⚠️ این خط را از کامنت دربیاور تا خودت ببینی خطا می‌دهد:
    // assert_send::<std::rc::Rc<u8>>();
    println!("  (Rc<u8> ارسال‌پذیر *نیست* — شمارندهٔ غیراتمیک دارد)");

    // ۴) تأیید ساختار با `debug_assert!` — در نسخهٔ debug فعال، در release حذف می‌شود
    let v = vec![1, 2, 3];
    debug_assert!(v.len() == 3, "طول بردار باید ۳ باشد");
    println!("  debug_assert: در حالت debug چک می‌شود، در release حذف (بدون هزینه)");

    // ۵) نام و مسیر نوع در زمان کامپایل با `std::any::type_name`
    println!("  type_name::<Vec<Option<u8>>>() = {}", std::any::type_name::<Vec<Option<u8>>>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn repetition_macros_work() {
        assert_eq!(my_vec![1, 2, 3], vec![1, 2, 3]);
        assert_eq!(my_vec![7; 3], vec![7, 7, 7]);
        let empty: Vec<i32> = my_vec![];
        assert!(empty.is_empty());
        assert_eq!(sum_all!(1, 2, 3), 6);
    }

    #[test]
    fn getter_macro_generated_methods() {
        let m = Metrics { requests: 9, errors: 1 };
        assert_eq!(m.requests(), 9);
        assert_eq!(m.errors(), 1);
    }

    #[test]
    fn url_macro_builds_paths() {
        assert_eq!(url!("x.com"), "https://x.com");
        assert_eq!(url!("x.com", "/a"), "https://x.com/a");
        assert_eq!(url!("x.com", "a", "b"), "https://x.com/a/b");
    }
}
