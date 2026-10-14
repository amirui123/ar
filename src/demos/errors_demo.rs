//! # ⚠️ دموی خطاها — «هرگز نترس، همیشه مدیریت کن»
//!
//! قانون طلایی Rust:
//!
//! | موقعیت | ابزار |
//! |--------|-------|
//! | خطای قابل انتظار (فایل نیست، ورودی بد) | `Result<T, E>` |
//! | باگ برنامه‌نویس (فرض نقض‌شده) | `panic!` |
//! | نمونه‌سازی سریع (اسکریپت/تست) | `unwrap()` |
//!
//! و مهم‌ترین جملهٔ این فصل: **کتابخانه نباید پنیک کند.**

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) Result — خطا به‌عنوان مقدار"));
    results();

    println!("\n{}", color::bold("۲) `?` و تبدیل خودکار خطا با From"));
    question_mark();

    println!("\n{}", color::bold("۳) خطای سفارشی با زنجیرهٔ علت"));
    custom_errors();

    println!("\n{}", color::bold("۴) ترکیب‌کننده‌ها (combinators) به‌جای match"));
    combinators();

    println!("\n{}", color::bold("۵) پنیک — کجا مجاز است و کجا نه"));
    panics();

    println!("\n{}", color::bold("۶) الگوهای حرفه‌ای"));
    pro_patterns();
}

// ═══════════════════════════════════════════════════════════════════════════
/// تقسیم با خطای نوع‌دار.
fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        // `return Err(...)` به‌جای پنیک: به فراخوان تصمیم می‌دهیم
        return Err(MathError::DivisionByZero);
    }
    Ok(a / b)
}

/// خطاهای ریاضی.
#[derive(Debug, PartialEq)]
pub enum MathError {
    /// تقسیم بر صفر
    DivisionByZero,
    /// جذر عدد منفی
    NegativeSqrt(f64),
    /// سرریز
    Overflow,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "تقسیم بر صفر مجاز نیست"),
            MathError::NegativeSqrt(v) => write!(f, "جذر عدد منفی ({v}) تعریف نشده"),
            MathError::Overflow => write!(f, "نتیجه از محدودهٔ نوع بیرون زد"),
        }
    }
}

// پیاده‌سازی استاندارد خطا → سازگاری با Box<dyn Error> و anyhow و …
impl Error for MathError {}

fn results() {
    match divide(10.0, 4.0) {
        Ok(v) => println!("  10/4 = {v}"),
        Err(e) => println!("  خطا: {e}"),
    }

    match divide(1.0, 0.0) {
        Ok(v) => println!("  نتیجه: {v}"),
        Err(e) => println!("  خطا (به‌جای پنیک): {e}"),
    }

    // `if let` وقتی فقط یک حالت مهم است
    if let Err(e) = divide(1.0, 0.0) {
        println!("  با if-let: {e}");
    }

    // `unwrap_or` / `unwrap_or_else` / `unwrap_or_default`
    println!("  با fallback: {}", divide(1.0, 0.0).unwrap_or(f64::NAN));
    println!("  با پیش‌فرض: {}", divide(1.0, 0.0).unwrap_or_default());
    println!("  با محاسبهٔ جایگزین: {}", divide(1.0, 0.0).unwrap_or_else(|_| 1.0 / f64::EPSILON));

    // `ok()` و `err()` برای تبدیل به Option
    println!("  ok() = {:?}، err() = {:?}", divide(4.0, 2.0).ok(), divide(4.0, 0.0).err());

    // `Result<Option<T>>` یا `Option<Result<T>>` — هرکدام معنی خودش را دارد!
    let from_map: Result<Option<i32>, &str> = Ok(Some(1));
    let from_opt: Option<Result<i32, &str>> = Some(Ok(1));
    println!("  Result<Option> = {from_map:?}  در برابر  Option<Result> = {from_opt:?}");
    println!("  ↑ این دو یکی نیستند: اولی «خطای کل عملیات»، دومی «نبودِ مقدار درون عملیات موفق»");
}

// ═══════════════════════════════════════════════════════════════════════════
/// خطای سطح اپلیکیشن.
#[derive(Debug)]
enum ConfigError {
    /// فایل خوانده نشد
    Io(std::io::Error),
    /// عدد نامعتبر
    Parse(ParseIntError),
    /// مقدار خارج از محدوده
    OutOfRange {
        /// نام فیلد
        field: &'static str,
        /// مقدار
        value: i64,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "خطای خواندن پیکربندی: {e}"),
            ConfigError::Parse(e) => write!(f, "عدد نامعتبر در پیکربندی: {e}"),
            ConfigError::OutOfRange { field, value } => {
                write!(f, "مقدار «{field}» ({value}) خارج از محدودهٔ مجاز است")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // 🌟 این متد، «زنجیرهٔ علت» را می‌سازد
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Parse(e) => Some(e),
            ConfigError::OutOfRange { .. } => None,
        }
    }
}

// 🔑 با پیاده‌سازی From، عملگر `?` تبدیل‌ها را خودکار انجام می‌دهد.
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::Parse(e)
    }
}

/// خواندن پیکربندی از متن (بدون فایل، برای تست).
fn parse_config(text: &str) -> Result<u16, ConfigError> {
    // این خط می‌تواند هم خطای Parse بدهد هم OutOfRange
    let port: i64 = text.trim().parse()?; // `?` خودش به ConfigError تبدیل می‌کند

    if !(1024..=65535).contains(&port) {
        return Err(ConfigError::OutOfRange { field: "port", value: port });
    }
    Ok(port as u16)
}

fn question_mark() {
    println!("  parse_config(\"8080\")   = {:?}", parse_config("8080"));
    println!("  parse_config(\"99999\")  = {:?}", parse_config("99999").unwrap_err());
    println!("  parse_config(\"abc\")    = {:?}", parse_config("abc").unwrap_err());

    // 🧠 نکتهٔ مهم: `?` *تمام* خطاها را با From تبدیل می‌کند — بدون boilerplate.
    //    در Rust 2024 می‌شود `?` را روی Option هم در تابع Result به کار برد
    //    با `.ok_or(...)` یا `ok_or_else`.
    let opt: Option<i32> = Some(5);
    let res: Result<i32, &str> = opt.ok_or("نبود مقدار");
    println!("  Option → Result با ok_or: {res:?}");

    // و برعکس:
    let back: Option<i32> = res.ok();
    println!("  Result → Option با ok: {back:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
/// خطای لایهٔ سرویس که خطای پایین‌تر را «می‌پیچد».
#[derive(Debug)]
struct ServiceError {
    /// عملیات
    operation: String,
    /// علت
    cause: ConfigError,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "شکست در «{}»", self.operation)
    }
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

impl From<ConfigError> for ServiceError {
    fn from(cause: ConfigError) -> Self {
        ServiceError { operation: "راه‌اندازی سرور".to_string(), cause }
    }
}

fn custom_errors() {
    let result: Result<u16, ServiceError> = parse_config("99999").map_err(ServiceError::from);
    let err = result.unwrap_err();

    // چاپ زنجیرهٔ خطاها — همان کاری که anyhow/eyre خودکار انجام می‌دهند
    println!("  خطا: {err}");
    let mut src: Option<&(dyn Error + 'static)> = err.source();
    let mut depth = 1;
    while let Some(cause) = src {
        println!("  {} علت {depth}: {cause}", "↳".repeat(depth));
        src = cause.source();
        depth += 1;
    }

    // 🧰 ابزار همه‌کاره: Box<dyn Error>
    //    خوب برای برنامه‌های اجرایی («همه نوع خطا را می‌پذیرم»)
    //    بد برای کتابخانه‌ها («کاربر نمی‌تواند نوع خطا را تشخیص دهد»)
    // (توجه: `map_err(Box::from)` مبهم است؛ کامپایلر نمی‌داند کدام Box را می‌خواهی
    //  — پس با یک بستار نوع را صریح می‌کنیم. این هم از «تلاش آموزش کامپایلر» است!)
    let boxed: Result<u16, Box<dyn Error>> =
        parse_config("70000").map_err(|e| -> Box<dyn Error> { Box::new(e) });
    println!("  Box<dyn Error>: {}", boxed.unwrap_err());

    // جعبه با متن دلخواه:
    let adhoc: Result<(), Box<dyn Error>> = Err("یک خطای سریع و بی‌نوع".into());
    println!("  Box از &str: {}", adhoc.unwrap_err());
}

// ═══════════════════════════════════════════════════════════════════════════
fn combinators() {
    let ok: Result<i32, &str> = Ok(4);
    let bad: Result<i32, &str> = Err("خراب");

    println!("  map        : {:?}", ok.map(|x| x * 10));
    println!("  and_then   : {:?}", ok.and_then(|x| if x > 3 { Ok(x) } else { Err("کوچک") }));
    println!("  or_else    : {:?}", bad.or_else(|_| Ok::<i32, &str>(-1)));
    println!("  unwrap_or  : {}", bad.unwrap_or(0));
    println!("  map_err    : {:?}", bad.map_err(|e| format!("ترجمه‌شده: {e}")));
    println!("  is_ok_and  : {:?}", ok.is_ok_and(|x| x > 1));
    println!("  transpose  : {:?}", Some(Ok::<i32, &str>(1)).transpose());
    println!("  inspect    : {:?}", ok.inspect(|x| println!("    (inspect دید {x})")));

    // `and_then` برای انتقال بین مراحل پرهزینه است:
    fn parse_port(s: &str) -> Result<u16, String> {
        s.parse::<u16>().map_err(|e| format!("عدد نامعتبر: {e}"))
    }
    fn to_socket(port: u16) -> Result<std::net::SocketAddr, String> {
        format!("127.0.0.1:{port}")
            .parse()
            .map_err(|e| format!("آدرس نامعتبر: {e}"))
    }
    let addr = parse_port("8080").and_then(to_socket);
    println!("  زنجیرهٔ and_then: {addr:?}");

    // ⚠️ ضدالگو: unwrap پشت unwrap — کتابخانه هرگز این کار را نکند
    println!("  ❌ parse_port(\"x\").unwrap().to_string() → پنیک!");
    println!("  ✅ let port = parse_port(\"x\")?;  → خطا به فراخوان برمی‌گردد");
}

// ═══════════════════════════════════════════════════════════════════════════
fn panics() {
    // ✅ پنیک درست: نقض یک «ثابت» (invariant) که باگ برنامه‌نویس است.
    // ✅ پنیک درست: در تست‌ها.
    // ❌ پنیک غلط: روی ورودی کاربر یا هر چیزی که در دنیای واقعی ممکن است رخ دهد.

    // `expect` پیام بهتری از `unwrap` می‌دهد → همیشه expect را ترجیح بده
    let parsed: i32 = "42".parse().expect("عدد ۴۲ باید قابل پارس باشد — این یک باگ است");

    println!("  parsed = {parsed}");

    // پیام‌های صریح‌تر برای «غیرممکن‌ها»
    fn never_reached(x: u8) -> &'static str {
        match x {
            0 => "صفر",
            1 => "یک",
            // ⚠️ پنیک با پیام — اینجا مجاز است چون باگ است اگر رخ دهد
            other => unreachable!("مقدار {other} با منطق ما در تضاد است"),
        }
    }
    println!("  never_reached(1) = {}", never_reached(1));

    // `todo!` و `unimplemented!` — با تایپ‌سیفتی کامل (برای توسعه تدریجی)
    fn future_feature() -> Result<(), String> {
        // todo!("در نسخهٔ بعدی پیاده‌سازی می‌شود")
        Ok(())
    }
    println!("  todo!/unimplemented! هم نوع درست برمی‌گردانند → کد کامپایل می‌شود: {:?}", future_feature());

    // 🔒 گرفتن پنیک با catch_unwind — فقط برای مرزهای خاص (مثل ترد یا FFI)
    // (نیاز به panic = "unwind" دارد — ما در Cargo.toml همین را گذاشتیم)
    let caught = std::panic::catch_unwind(|| {
        // این پنیک را می‌گیریم؛ پیام پیش‌فرض آن چاپ نشود:
        panic!("این پنیک عمدی است و گرفته می‌شود");
    });
    println!("  catch_unwind → خطا گرفته شد؟ {}", caught.is_err());
    println!("  ⚠️ catch_unwind برای مدیریت خطای *عادی* نیست؛ فقط مرزهای بحرانی");

    // ⚠️ نکتهٔ مهم: پنیک در Drop باعث «پنیک در پنیک» می‌شود → فرایند می‌میرد
    println!("  (یادت باشد: هرگز در Drop پنیک نکن — به docs/14 نگاه کن)");
}

// ═══════════════════════════════════════════════════════════════════════════
fn pro_patterns() {
    // ۱) «پارس کن، اعتبارسنجی نکن» (Parse, don't validate)
    //    اگر نوعی ساخته شد، یعنی معتبر است. (الگوی PostalCode در دموی generics)

    // ۲) خطا را با «زمینه» غنی کن — الگویی که anyhow::Context ارائه می‌دهد
    fn load(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|e| format!("خواندن فایل «{path}» شکست خورد: {e}"))
    }
    println!("  ۱) زمینه‌دار: {}", load("/مسیر/ناموجود").unwrap_err());

    // ۳) `collect` روی Result: یا همه موفق، یا اولین خطا
    let all: Result<Vec<i32>, _> = ["1", "2", "x"].iter().map(|s| s.parse::<i32>()).collect();
    println!("  ۲) collect روی Result: {all:?}  ← کل عملیات اتمی می‌شود");

    // ۴) `main() -> Result<..>`: ترفندی که چاپ خطا را خودکار می‌کند
    //    (در باینری به‌جای panic، پیام Debug چاپ می‌شود)
    fn main_like() -> Result<(), Box<dyn Error>> {
        parse_config("80").map_err(|e| -> Box<dyn Error> { Box::new(e) })?;
        Ok(())
    }
    println!("  ۳) main-like: {}", main_like().unwrap_err());

    // ۵) تفکیک «خطای قابل بازیابی» از «خطای کشنده» با نوع
    #[derive(Debug)]
    enum FetchError {
        /// موقت — دوباره تلاش کن
        Timeout,
        /// دائمی — تلاش نکن
        NotFound,
    }
    fn should_retry(e: &FetchError) -> bool {
        matches!(e, FetchError::Timeout)
    }
    println!("  ۴) Timeout قابل تلاش مجدد: {}", should_retry(&FetchError::Timeout));
    println!("     NotFound قابل تلاش مجدد: {}", should_retry(&FetchError::NotFound));

    // ۶) `expect` در جای «انتظار داریم هرگز رخ ندهد» با مستندسازی
    println!("  ۵) دو نوع خطا داشته باش: «انتظاری» و «کشنده» — و در API جاهایشان را ثبت کن");
    println!("  📚 مطالعه: `anyhow` (برای برنامه) و `thiserror` (برای کتابخانه)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn divide_never_panics() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
        assert_eq!(divide(1.0, 0.0), Err(MathError::DivisionByZero));
    }

    #[test]
    fn config_errors_convert_automatically() {
        assert_eq!(parse_config("8080").unwrap(), 8080);
        assert!(matches!(parse_config("abc"), Err(ConfigError::Parse(_))));
        assert!(matches!(parse_config("70000"), Err(ConfigError::OutOfRange { .. })));
    }

    #[test]
    fn error_chain_has_source() {
        let e = ServiceError::from(ConfigError::OutOfRange { field: "x", value: 1 });
        assert!(e.source().is_some());
        assert!(e.to_string().contains("راه‌اندازی"));
    }

    #[test]
    fn collect_result_is_atomic() {
        let ok: Result<Vec<i32>, String> = ["1", "2"].iter().map(|s| s.parse::<i32>().map_err(|e| e.to_string())).collect();
        assert_eq!(ok, Ok(vec![1, 2]));
        let bad: Result<Vec<i32>, String> = ["1", "x"].iter().map(|s| s.parse::<i32>().map_err(|e| e.to_string())).collect();
        assert!(bad.is_err());
    }
}
