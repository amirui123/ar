//! # 🧱 مدل داده — struct، enum، صفت‌های مشتق‌شده
//!
//! این ماژول «مدل دامنه» (domain model) پروژه است: تسک، اولویت و شناسه.
//! هدف آموزشی: دیدن اینکه چطور با ابزارهای *نوع* (type system) می‌توان
//! جلوی کلی باگ را همان لحظهٔ کامپایل گرفت.
//!
//! ## سه ایدهٔ بزرگ در این فایل
//! 1. **Newtype** — نوع جدید دور یک نوع موجود تا شناسه‌ها با هم قاطی نشوند.
//! 2. **enum با داده** — «اولویت» سه حالت دارد و کامپایلر مطمئن می‌شود
//!    هیچ‌وقت حالت چهارمی را فراموش نکنی (`match` جامع).
//! 3. **derive** — تولید خودکار کد تکراری (Debug, Clone, PartialEq, Hash, Ord …).

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::{AppError, Result};

// ═══════════════════════════════════════════════════════════════════════════
//  ۱) Newtype — شناسهٔ تسک
// ═══════════════════════════════════════════════════════════════════════════
/// شناسهٔ یکتای تسک (newtype).
///
/// چرا `struct TaskId(u64)` و نه فقط `u64`؟
/// چون در یک تابع مثل `fn complete(user_id: u64, task_id: u64)` به‌راحتی
/// آرگومان‌ها را جابه‌جا می‌کنی و کامپایلر چیزی نمی‌گوید. با newtype:
///
/// ```compile_fail
/// # use ar::model::TaskId;
/// fn complete(user: u64, task: TaskId) {}
/// // این خط *کامپایل نمی‌شود* — خیال ما راحت است:
/// complete(TaskId(7), 9);
/// ```
///
/// 🔑 **ترفند:** `#[repr(transparent)]` می‌گوید این ساختار در حافظه *دقیقاً*
/// مثل `u64` است → صفر هزینه در زمان اجرا (zero-cost abstraction).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

impl TaskId {
    /// شمارهٔ خام داخل newtype.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0) // هر TaskId حالا مثل «#12» چاپ می‌شود
    }
}

impl From<u64> for TaskId {
    fn from(v: u64) -> Self {
        TaskId(v)
    }
}

impl FromStr for TaskId {
    type Err = AppError;

    /// پارس شناسه از ورودی کاربر. هم «12» و هم «#12» پذیرفته می‌شود.
    /// این پیاده‌سازی `FromStr` باعث می‌شود `"12".parse::<TaskId>()` کار کند.
    fn from_str(s: &str) -> Result<Self> {
        let cleaned = s.trim().trim_start_matches('#');
        cleaned
            .parse::<u64>()
            .map(TaskId)
            .map_err(|e| AppError::parse_int(format!("شناسهٔ تسک «{s}»"), e))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۲) enum — اولویت
// ═══════════════════════════════════════════════════════════════════════════
/// اولویت تسک.
///
/// نکته: ترتیب واریانت‌ها را از کم به زیاد می‌چینیم تا `#[derive(PartialOrd, Ord)]`
/// دقیقاً همان ترتیب منطقی را بدهد (کم‌ترین = Low).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// پایین
    Low,
    /// معمولی (پیش‌فرض)
    Normal,
    /// بالا
    High,
    /// فوری
    Urgent,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl Priority {
    /// همهٔ حالت‌ها به‌ترتیب — برای پیمایش در جدول/آمار.
    ///
    /// 🔑 **ترفند:** به‌جای نوشتن دستی آرایه، از `std::array` و `IntoIterator`
    /// استفاده می‌کنیم؛ در Rust مدرن، پیمایش آرایه با مقدار (by value) هم ممکن است.
    pub const ALL: [Priority; 4] = [Priority::Low, Priority::Normal, Priority::High, Priority::Urgent];

    /// نام انگلیسی (برای JSON و ذخیره‌سازی).
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        }
    }

    /// برچسب فارسی (برای نمایش).
    pub fn label(self) -> &'static str {
        match self {
            Priority::Low => "پایین",
            Priority::Normal => "معمولی",
            Priority::High => "بالا",
            Priority::Urgent => "فوری",
        }
    }

    /// آیکن رنگی.
    pub fn icon(self) -> &'static str {
        match self {
            Priority::Low => "▁",
            Priority::Normal => "▄",
            Priority::High => "▆",
            Priority::Urgent => "█",
        }
    }

    /// کد رنگ ANSI برای ترمینال.
    pub fn color_code(self) -> &'static str {
        match self {
            Priority::Low => crate::color::DIM,
            Priority::Normal => crate::color::BLUE,
            Priority::High => crate::color::YELLOW,
            Priority::Urgent => crate::color::RED,
        }
    }

    /// «وزن» عددی برای مرتب‌سازی/آمار (بالاتر = مهم‌تر).
    pub fn weight(self) -> u32 {
        match self {
            Priority::Low => 1,
            Priority::Normal => 2,
            Priority::High => 4,
            Priority::Urgent => 8,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let painted = crate::color::paint(self.label(), self.color_code());
        // `{:<8}` یعنی چپ‌چین در ۸ کاراکتر. اما رنگ، کاراکترهای نامرئی اضافه می‌کند؛
        // پس اگر رنگ فعال باشد، خودمان فاصله‌گذاری می‌کنیم (ترفند جدول‌سازی).
        if crate::color::enabled() {
            write!(f, "{painted}")
        } else {
            write!(f, "{:<8}", self.label())
        }
    }
}

impl FromStr for Priority {
    type Err = AppError;

    /// پارس اولویت: هم انگلیسی («high», «h») هم فارسی («بالا»).
    fn from_str(s: &str) -> Result<Self> {
        // `to_lowercase` یک String جدید می‌سازد؛ برای ورودی کوچک ضرری ندارد.
        match s.trim().to_lowercase().as_str() {
            "l" | "low" | "پایین" => Ok(Priority::Low),
            "n" | "normal" | "معمولی" | "" => Ok(Priority::Normal),
            "h" | "high" | "بالا" => Ok(Priority::High),
            "u" | "urgent" | "فوری" => Ok(Priority::Urgent),
            other => Err(AppError::BadInput(format!(
                "اولویت «{other}» ناشناخته است (low|normal|high|urgent)"
            ))),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۳) struct — تسک
// ═══════════════════════════════════════════════════════════════════════════
/// یک تسک (کار).
///
/// فیلدها `pub` هستند چون این یک ساختار دادهٔ ساده است؛ در کتابخانهٔ واقعی
/// بهتر است فیلدها خصوصی باشند و از طریق متدها دسترسی بدهیم
/// (راهنمای API زبان: `C-STRUCT-PRIVATE`) — ماژول `docs/13-tricks.md` را ببین.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// شناسهٔ یکتا
    pub id: TaskId,
    /// عنوان تسک
    pub title: String,
    /// اولویت
    pub priority: Priority,
    /// برچسب‌ها (بدون تکرار، مرتب‌شده هنگام ساخت)
    pub tags: Vec<String>,
    /// انجام‌شده؟
    pub done: bool,
    /// زمان ساخت (ثانیه از مبدأ یونیکس، UTC)
    pub created_at: u64,
    /// زمان انجام (اگر انجام شده باشد)
    pub done_at: Option<u64>,
}

impl Task {
    /// ساخت تسک تازه. زمان ساخت، «الان» است.
    ///
    /// `title: impl Into<String>` یک **ترفند ارگونومی** است: هم `&str` می‌پذیرد
    /// هم `String`؛ پس نه فراخوان مجبور به `to_string()` می‌شود، نه ما کپی اضافه می‌کنیم.
    pub fn new(id: TaskId, title: impl Into<String>, priority: Priority, mut tags: Vec<String>) -> Self {
        // ترفند: برچسب‌ها را پاک‌سازی و بی‌تکرار کنیم
        for t in &mut tags {
            *t = t.trim().to_lowercase();
        }
        tags.retain(|t| !t.is_empty()); // حذف خالی‌ها — `retain` مثل filter ولی «درجا»
        tags.sort();
        tags.dedup(); // چون مرتب شده، `dedup` کافی است (بدون HashSet)

        Task {
            id,
            title: title.into(),
            priority,
            tags,
            done: false,
            created_at: now_unix(),
            done_at: None,
        }
    }

    /// تسک را انجام‌شده/انجام‌نشده می‌کند و زمان را درست ثبت می‌کند.
    /// خروجی: آیا تغییری رخ داد؟ (الگوی «idempotent»)
    pub fn set_done(&mut self, done: bool) -> bool {
        if self.done == done {
            return false; // قبلاً همین وضعیت بود → کار بی‌فایده نکن
        }
        self.done = done;
        self.done_at = if done { Some(now_unix()) } else { None };
        true
    }

    /// آیا این تسک برچسب مشخصی دارد؟
    pub fn has_tag(&self, tag: &str) -> bool {
        // چون tags مرتب است، می‌توانستیم binary_search بزنیم؛ ولی اینجا تعداد کم است
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    /// آیا عنوان شامل جست‌وجو هست؟ (بی‌توجه به بزرگی/کوچکی حروف)
    pub fn matches(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.title.to_lowercase().contains(&needle)
            || self.tags.iter().any(|t| t.contains(&needle))
    }

    /// مدت زمان تا انجام‌شدن (ثانیه) — اگر انجام شده باشد.
    pub fn time_to_done(&self) -> Option<u64> {
        // الگوی `?` روی Option: اگر None بود، کل تابع None می‌دهد
        let done_at = self.done_at?;
        Some(done_at.saturating_sub(self.created_at)) // saturating_* = بدون سرریز/منفی
    }

    /// مقایسهٔ «فوریت» برای مرتب‌سازی.
    ///
    /// 🔑 **ترفند:** با `Ordering` به‌جای bool، `sort_by` خوانا می‌شود و
    /// می‌توانیم `.reverse()` هم بزنیم.
    pub fn urgency_cmp(&self, other: &Self) -> Ordering {
        // اولویت بالا اول، بعد قدیمی‌ترها اول (FIFO در هر اولویت)
        other
            .priority
            .cmp(&self.priority)          // عمداً برعکس (urgent اول)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = if self.done { "✔" } else { "☐" };
        write!(f, "{mark} {} {}", self.id, self.title)?;
        if !self.tags.is_empty() {
            write!(f, " [{}]", self.tags.join(", "))?;
        }
        Ok(())
    }
}

/// زمان «الان» به ثانیه از مبدأ یونیکس.
///
/// نکته‌ها:
/// * `SystemTime::now()` ممکن است قبل از ۱۹۷۰ باشد (ساعت سیستم خراب) → با
///   `unwrap_or_default()` به‌جای `unwrap()` برنامه را نمی‌کشیم.
/// * `duration_since` خروجی‌اش `Result` است، پس الگوی «fallback بی‌خطر» می‌خواهیم.
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// تبدیل ثانیهٔ یونیکس به فرمت خوانا مثل «1405/… » یا «3 دقیقه پیش».
///
/// ما وابستگی نداریم (chrono نیست!) پس تقویم را ساده نگه می‌داریم: زمان نسبی.
pub fn humanize_age(seconds_ago: u64) -> String {
    // الگوی «آستانه‌ها» با match و محدوده‌ها — بسیار خوانا
    match seconds_ago {
        0..=59 => "همین حالا".to_string(),
        60..=3599 => format!("{} دقیقه پیش", seconds_ago / 60),
        3600..=86_399 => format!("{} ساعت پیش", seconds_ago / 3600),
        86_400..=2_591_999 => format!("{} روز پیش", seconds_ago / 86_400),
        s => format!("{} ماه پیش", s / 2_592_000),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  تست‌ها
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtype_parses_with_and_without_hash() {
        assert_eq!("#12".parse::<TaskId>().unwrap(), TaskId(12));
        assert_eq!("12".parse::<TaskId>().unwrap(), TaskId(12));
        assert!("abc".parse::<TaskId>().is_err());
    }

    #[test]
    fn tags_are_normalized_and_deduped() {
        let t = Task::new(
            TaskId(1),
            "تست",
            Priority::Normal,
            vec!["  Rust ".into(), "rust".into(), "".into(), "CLI".into()],
        );
        assert_eq!(t.tags, vec!["cli".to_string(), "rust".to_string()]);
    }

    #[test]
    fn set_done_is_idempotent() {
        let mut t = Task::new(TaskId(1), "x", Priority::Low, vec![]);
        assert!(t.set_done(true));   // بار اول تغییر کرد
        assert!(!t.set_done(true));  // بار دوم نه
        assert!(t.done_at.is_some());
        assert!(t.set_done(false));
        assert!(t.done_at.is_none());
    }

    #[test]
    fn priority_ordering_is_logical() {
        assert!(Priority::Urgent > Priority::High);
        assert!(Priority::Low < Priority::Normal);
        assert_eq!("فوری".parse::<Priority>().unwrap(), Priority::Urgent);
        assert_eq!("H".parse::<Priority>().unwrap(), Priority::High);
        assert!("هیچ".parse::<Priority>().is_err());
    }

    #[test]
    fn urgency_sorts_by_priority_then_age() {
        let mut a = Task::new(TaskId(1), "a", Priority::Low, vec![]);
        a.created_at = 100;
        let mut b = Task::new(TaskId(2), "b", Priority::Urgent, vec![]);
        b.created_at = 200;
        let mut list = vec![a, b];
        list.sort_by(Task::urgency_cmp);
        assert_eq!(list[0].id, TaskId(2)); // فوری اول باید بیاید
    }

    #[test]
    fn humanize_is_sane() {
        assert_eq!(humanize_age(10), "همین حالا");
        assert_eq!(humanize_age(120), "2 دقیقه پیش");
        assert_eq!(humanize_age(7200), "2 ساعت پیش");
    }
}
