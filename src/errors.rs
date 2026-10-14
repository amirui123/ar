//! # 📛 خطاها — قلبِ یک برنامهٔ سالم
//!
//! در Rust «خطا» یک مقدار است، نه یک استثنا (exception). یعنی خطا هم مثل بقیهٔ
//! داده‌ها در امضای تابع دیده می‌شود: `fn f() -> Result<T, E>`.
//!
//! دو نوع خطا داریم:
//!
//! | نوع | چه وقت | ابزار |
//! |-----|--------|-------|
//! | **قابل بازیابی** (recoverable) | فایل پیدا نشد، ورودی بد، شبکه قطع | `Result<T, E>` |
//! | **غیرقابل بازیابی** (unrecoverable) | باگ منطقی، فرض نقض‌شده | `panic!` |
//!
//! قاعدهٔ تجربی:
//! * کتابخانه‌ها هرگز نباید `panic!` کنند → `Result` برگردانند.
//! * برنامه‌های اجرایی مجازند در موارد «خرابی واقعی» پنیک کنند.
//!
//! ## الگوی این پروژه: یک enum خطا برای کل برنامه
//! به‌جای اینکه هر تابع خطای خودش را بسازد، یک `enum AppError` داریم و با پیاده‌سازی
//! `From` برای خطاهای دیگر (مثل `std::io::Error`)، عملگر `?` تبدیل‌ها را خودکار می‌کند.

use std::fmt;
use std::io;
use std::num::ParseIntError;

/// نوع `Result` اختصاصی پروژه.
///
/// 🔑 **ترفند:** وقتی `Result<T, AppError>` را دائماً می‌نویسیم، اسمش زیاد می‌شود.
/// یک «نام مستعار» (type alias) با مقدار پیش‌فرض خطا می‌سازیم تا کد تمیزتر شود.
/// این الگو در خود std هم هست (`std::io::Result<T>` دقیقاً همین کار را می‌کند).
pub type Result<T> = std::result::Result<T, AppError>;

/// همهٔ خطاهای ممکن برنامه.
///
/// چرا `enum` و نه یک ساختار با رشتهٔ خطا؟
/// چون با `match` می‌توان *نوع* خطا را تشخیص داد و رفتار متفاوت نشان داد؛
/// مثلاً برای «چیزی پیدا نشد» کد خروج (exit code) برابر ۲ بدهیم.
#[derive(Debug)]
pub enum AppError {
    /// خطای ورودی/خروجی سیستم (فایل، شبکه) به‌همراه «زمینه» (context).
    ///
    /// الگوی `{ ctx, src }` **خیلی مهم** است: خطای اصلی را دور نمی‌ریزیم
    /// (`source()` را ببین) ولی به کاربر می‌گوییم *کدام عملیات* شکست خورد.
    Io {
        /// چه کاری انجام می‌شد؟ مثلاً: "خواندن فایل `data/tasks.json`"
        ctx: String,
        /// خطای اصلی سیستم‌عامل
        src: io::Error,
    },

    /// تبدیل رشته به عدد شکست خورد (با زمینهٔ کاربرد).
    ParseInt {
        /// نمونۀ کاربرد: "پارس شناسهٔ تسک"
        ctx: String,
        /// خطای اصلی
        src: ParseIntError,
    },

    /// خطای پارس JSON (ماژول [`crate::json`]).
    Json(crate::json::JsonError),

    /// تسکی با این شناسه وجود ندارد.
    TaskNotFound(u64),

    /// ورودی کاربر نامعتبر بود (پیام دوستانه برای نمایش).
    BadInput(String),

    /// استفادهٔ اشتباه از کامند خط فرمان (به‌همراه راهنما).
    Usage(String),
}

impl AppError {
    /// ساخت خطای I/O با زمینهٔ دلخواه (idempotent و زنجیره‌پذیر).
    pub fn io(ctx: impl Into<String>, src: io::Error) -> Self {
        AppError::Io { ctx: ctx.into(), src }
    }

    /// ساخت خطای پارس عدد با زمینهٔ دلخواه.
    pub fn parse_int(ctx: impl Into<String>, src: ParseIntError) -> Self {
        AppError::ParseInt { ctx: ctx.into(), src }
    }

    /// کد خروج پیشنهادی برای این خطا (قرارداد رایج یونیکس).
    pub fn exit_code(&self) -> i32 {
        match self {
            // 1 = خطای عمومی
            AppError::Io { .. } | AppError::ParseInt { .. } | AppError::Json(_) => 1,
            // 2 = استفادهٔ نادرست از کامند (مثل رفتار `grep` و `bash`)
            AppError::Usage(_) | AppError::BadInput(_) => 2,
            // 3 = «چیزی که دنبالش بودیم نبود» (اختصاصی خودمان)
            AppError::TaskNotFound(_) => 3,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Display — پیام انسانی
// ─────────────────────────────────────────────────────────────────────────────
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // الگوی `{ctx}` = روش کوتاه نوشتن `self.ctx` در فرمت‌رشته‌ها (Rust ≥ 1.58)
            AppError::Io { ctx, src } => write!(f, "خطای فایل/شبکه در «{ctx}»: {src}"),
            AppError::ParseInt { ctx, src } => write!(f, "عدد نامعتبر در «{ctx}»: {src}"),
            AppError::Json(e) => write!(f, "{e}"),
            AppError::TaskNotFound(id) => {
                write!(f, "تسکی با شناسهٔ #{id} پیدا نشد")
            }
            AppError::BadInput(msg) => write!(f, "ورودی نامعتبر: {msg}"),
            AppError::Usage(msg) => write!(f, "{msg}"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  std::error::Error — پیاده‌سازی «قرارداد» خطاها
// ─────────────────────────────────────────────────────────────────────────────
//  نکتهٔ حرفه‌ای: `source()` زنجیرهٔ علت‌ها را می‌سازد. ابزارهایی مثل anyhow/eyre
//  با همین زنجیره، خطاها را لایه‌لایه چاپ می‌کنند:
//      خطا: خواندن فایل data/tasks.json شکست خورد
//      علت: No such file or directory (os error 2)
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io { src, .. } => Some(src),
            AppError::ParseInt { src, .. } => Some(src),
            AppError::Json(e) => Some(e),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  From — «ترفندِ» عملگر `?`
// ─────────────────────────────────────────────────────────────────────────────
//  عملگر `?` در واقع این کار را می‌کند:
//      match result { Ok(v) => v, Err(e) => return Err(From::from(e)) }
//  پس اگر From پیاده‌سازی شده باشد، `io::Error` خودکار به `AppError` تبدیل می‌شود
//  و کد ما پر از map_err نمی‌شود. ✨

impl From<io::Error> for AppError {
    fn from(src: io::Error) -> Self {
        AppError::Io { ctx: "عملیات فایل/شبکه".to_string(), src }
    }
}

impl From<ParseIntError> for AppError {
    fn from(src: ParseIntError) -> Self {
        AppError::ParseInt { ctx: "پارس عدد".to_string(), src }
    }
}

impl From<crate::json::JsonError> for AppError {
    fn from(e: crate::json::JsonError) -> Self {
        AppError::Json(e)
    }
}

// پیاده‌سازی From برای رشته‌ها، استفاده از `?` روی پیام‌های سادهٔ خودمان را راحت می‌کند.
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::BadInput(msg)
    }
}
impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::BadInput(msg.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  🪄 ماکروی کوچکِ پرکاربرد:  bail!
// ─────────────────────────────────────────────────────────────────────────────
/// از تابع بیرون بیا و یک خطای «ورودی نامعتبر» برگردان.
///
/// ```
/// use ar::bail;
/// fn check(x: i32) -> ar::Result<()> {
///     if x < 0 { bail!("x باید نامنفی باشد، ولی {x} است"); }
///     Ok(())
/// }
/// assert!(check(-1).is_err());
/// assert!(check(1).is_ok());
/// ```
///
/// 🔑 چرا ماکرو و نه تابع؟ چون باید `return` کند و *نوع بازگشتی تابع فراخوان* را
/// بفهمد؛ تابع معمولی نمی‌تواند این کار را بکند. این کلاس از ماکروها را
/// «statement macro» می‌نامند (الهام‌گرفته از `anyhow::bail!`).
#[macro_export]
macro_rules! bail {
    // الگوی «هرچیزی»: $($arg:tt)* یعنی هر توکنی، هر تعدادی → مستقیم به format! پاس می‌شود
    ($($arg:tt)*) => {
        return ::core::result::Result::Err(
            $crate::errors::AppError::BadInput(::std::format!($($arg)*))
        )
    };
}

/// مثل [`bail!`] ولی خطای «استفادهٔ نادرست از کامند» می‌سازد (کد خروج ۲).
#[macro_export]
macro_rules! usage_err {
    ($($arg:tt)*) => {
        return ::core::result::Result::Err(
            $crate::errors::AppError::Usage(::std::format!($($arg)*))
        )
    };
}

// ─────────────────────────────────────────────────────────────────────────────
//  تست‌های واحد — کنار خودِ کد (idiomatic در Rust)
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)] // این ماژول فقط هنگام `cargo test` کامپایل می‌شود → صفر هزینه در باینری نهایی
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_distinct() {
        assert_eq!(AppError::TaskNotFound(1).exit_code(), 3);
        assert_eq!(AppError::BadInput("x".into()).exit_code(), 2);
        let io_err = AppError::io("تست", io::Error::other("boom"));
        assert_eq!(io_err.exit_code(), 1);
    }

    #[test]
    fn source_chain_works() {
        // زنجیرهٔ علت باید خطای اصلی را نشان دهد
        let e = AppError::io("خواندن فایل", io::Error::other("زیرشکست"));
        let root = std::error::Error::source(&e).expect("باید علت وجود داشته باشد");
        assert!(root.to_string().contains("زیرشکست"));
    }

    #[test]
    fn bail_macro_returns_err() {
        fn f(x: i32) -> Result<i32> {
            if x == 0 {
                bail!("صفر مجاز نیست");
            }
            Ok(10 / x)
        }
        assert!(matches!(f(0), Err(AppError::BadInput(_))));
        assert_eq!(f(2).unwrap(), 5);
    }
}
