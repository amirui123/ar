//! # 💾 انباره (Store) — صفت‌ها، ارسال پویا، و ذخیره‌سازی امن
//!
//! این ماژول نشان می‌دهد چطور در Rust «چند پیاده‌سازی پشت یک واسط» بسازیم:
//!
//! ```text
//!           ┌───────────────────────┐
//!           │  trait Store          │   ← واسط (interface)
//!           └───────────┬───────────┘
//!            ┌──────────┴──────────┐
//!   ┌────────▼────────┐   ┌────────▼────────┐
//!   │  FileStore      │   │  MemoryStore    │
//!   │ (روی دیسک JSON) │   │ (فقط در حافظه) │
//!   └─────────────────┘   └─────────────────┘
//! ```
//!
//! و بالاتر از آن: `Box<dyn Store>` — یعنی «یک انباره، هر نوعی که باشه».
//!
//! ## درس‌های کلیدی
//! * `&self` در امضای صفت (نه `&mut self`) → چرا؟ چون [MemoryStore] با
//!   [`RefCell`](std::cell::RefCell) تغییرپذیری درونی دارد و لازم نیست
//!   همه‌جا `mut` پخش کنیم. این الگو در std هم هست (`Read for &File`).
//! * نوشتن **اتمیک** فایل (`tmp` + `rename`) → اگر برق برود، فایل خراب نمی‌شود.
//! * نسخه‌دار کردن قالب داده (`version`) → سازگاری با آینده.

use std::cell::RefCell;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, Result};
use crate::json::{self, Json};
use crate::model::{Priority, Task, TaskId};

/// نسخهٔ قالب فایل ذخیره‌سازی. هر وقت ساختار داده عوض شد، این عدد بالا می‌رود.
pub const SCHEMA_VERSION: i64 = 1;

/// یک «عکس لحظه‌ای» (snapshot) از کل دادهٔ برنامه.
///
/// 🔑 چرا ساختار جداگانه؟ چون [`Store`] باید بتواند `next_id` را هم ذخیره کند،
/// و اگر آن را جدا پاس می‌دادیم، هر پیاده‌سازی جدید باید یادش می‌ماند.
/// «داده‌های هم‌بسته را کنار هم بگذار» یک اصل طراحی است.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// شناسهٔ بعدی که تخصیص داده می‌شود
    pub next_id: u64,
    /// همهٔ تسک‌ها
    pub tasks: Vec<Task>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  صفت Store
// ═══════════════════════════════════════════════════════════════════════════
/// واسط ذخیره‌سازی.
///
/// نکتهٔ آموزشی: فقط دو متد اجباری داریم و یک متد پیش‌فرض ([`Store::describe`]).
/// متد پیش‌فرض یعنی «اگر پیاده‌ساز چیز خاصی نگفت، این را استفاده کن» —
/// روشی عالی برای کم کردن کار پیاده‌سازها.
pub trait Store {
    /// خواندن داده از انباره. اگر انباره خالی باشد، `Snapshot::default()`.
    ///
    /// # خطاها
    /// اگر خواندن یا پارس داده شکست بخورد.
    fn load(&self) -> Result<Snapshot>;

    /// نوشتن داده در انباره (کل داده، به‌صورت اتمیک).
    ///
    /// # خطاها
    /// اگر نوشتن شکست بخورد (دسترسی، پر بودن دیسک…).
    fn save(&self, snapshot: &Snapshot) -> Result<()>;

    /// توضیح کوتاه انباره برای پیام‌های کاربر.
    fn describe(&self) -> String {
        "انبارهٔ ناشناخته".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  پیاده‌سازی ۱: فایل
// ═══════════════════════════════════════════════════════════════════════════
/// انبارهٔ فایلی (JSON روی دیسک).
#[derive(Debug, Clone)]
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// ساخت انباره روی مسیر داده‌شده.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        FileStore { path: path.into() }
    }

    /// مسیر فایل.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// مسیر فایل موقت (کنار فایل اصلی تا `rename` اتمیک بماند).
    fn tmp_path(&self) -> PathBuf {
        let mut os = self.path.clone().into_os_string();
        os.push(".tmp"); // «tasks.json.tmp»
        PathBuf::from(os)
    }
}

impl Store for FileStore {
    fn load(&self) -> Result<Snapshot> {
        // الگوی مهم: «نبودن فایل» خطا نیست؛ یعنی شروع از صفر.
        if !self.path.exists() {
            return Ok(Snapshot::default());
        }

        let text = fs::read_to_string(&self.path).map_err(|e| {
            AppError::io(format!("خواندن فایل «{}»", self.path.display()), e)
        })?;

        // فایل خالی (مثلاً بعد از یک بار crash) → دادهٔ خالی
        if text.trim().is_empty() {
            return Ok(Snapshot::default());
        }

        let value = json::parse(&text)?;
        snapshot_from_json(&value)
    }

    fn save(&self, snapshot: &Snapshot) -> Result<()> {
        // ۱) اگر پوشهٔ والد وجود ندارد بساز (mkdir -p)
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::io(format!("ساخت پوشهٔ «{}»", parent.display()), e)
            })?;
        }

        // ۲) متن JSON را بساز
        let value = snapshot_to_json(snapshot);
        let text = value.to_pretty(2);

        // ۳) نوشتن اتمیک: اول در فایل موقت، بعد rename.
        //
        // 🧠 چرا؟ اگر وسط نوشتن مستقیم روی فایل اصلی برنامه بمیرد،
        //    فایل نیم‌نوشته و خراب می‌ماند. ولی `rename` در همان فایل‌سیستم
        //    «اتمیک» است: یا فایل قدیمی هست یا فایل کامل جدید. هیچ‌وقت نیمه‌کاره.
        let tmp = self.tmp_path();
        {
            let mut f = fs::File::create(&tmp).map_err(|e| {
                AppError::io(format!("ساخت فایل موقت «{}»", tmp.display()), e)
            })?;
            f.write_all(text.as_bytes()).map_err(|e| {
                AppError::io(format!("نوشتن در «{}»", tmp.display()), e)
            })?;
            // `sync_all` = اطمینان از رسیدن داده به دیسک (نه فقط کش سیستم‌عامل)
            f.sync_all().map_err(|e| AppError::io("همگام‌سازی با دیسک", e))?;
        }

        fs::rename(&tmp, &self.path).map_err(|e| {
            AppError::io(
                format!("جابه‌جایی «{}» به «{}»", tmp.display(), self.path.display()),
                e,
            )
        })?;

        Ok(())
    }

    fn describe(&self) -> String {
        format!("فایل «{}»", self.path.display())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  پیاده‌سازی ۲: حافظه (برای تست)
// ═══════════════════════════════════════════════════════════════════════════
/// انبارهٔ درون‌حافظه‌ای — برای تست‌ها.
///
/// 🔑 **چرا `RefCell`؟** صفت [`Store`] متدهایش `&self` می‌گیرند، ولی ما باید
/// داده را *تغییر* دهیم. `RefCell<T>` این اجازه را می‌دهد: قرض‌گیری را از
/// زمان کامپایل به زمان اجرا می‌برد (اگر هم‌زمان دو قرض متضاد بگیری، پنیک می‌کند).
/// این «تغییرپذیری درونی» (interior mutability) نام دارد.
#[derive(Debug, Default)]
pub struct MemoryStore {
    /// `Option<Snapshot>`: `None` یعنی «هیچ‌وقت ذخیره نشده» (مثل فایل ناموجود)
    inner: RefCell<Option<Snapshot>>,
}

impl MemoryStore {
    /// انبارهٔ خالی.
    pub fn new() -> Self {
        Self::default()
    }

    /// چند بار تا الان `save` صدا زده شده؟ (کمکی برای تست)
    pub fn is_populated(&self) -> bool {
        self.inner.borrow().is_some()
    }
}

impl Store for MemoryStore {
    fn load(&self) -> Result<Snapshot> {
        // `.borrow()` یک نگهبان (guard) برمی‌گرداند که وقتی از دامنه بیرون برود آزاد می‌شود.
        // `clone()` اینجا لازم است چون نگهبان را نمی‌توانیم برگردانیم (عمرش به `&self` بند است).
        Ok(self.inner.borrow().clone().unwrap_or_default())
    }

    fn save(&self, snapshot: &Snapshot) -> Result<()> {
        *self.inner.borrow_mut() = Some(snapshot.clone());
        Ok(())
    }

    fn describe(&self) -> String {
        "حافظهٔ موقت".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ترجمه: Task ↔ Json
// ═══════════════════════════════════════════════════════════════════════════
/// تبدیل کل عکس لحظه‌ای به JSON.
pub fn snapshot_to_json(snap: &Snapshot) -> Json {
    Json::obj([
        ("version", Json::from(SCHEMA_VERSION)),
        ("next_id", Json::from(snap.next_id)),
        (
            "tasks",
            Json::Arr(snap.tasks.iter().map(task_to_json).collect()),
        ),
    ])
}

/// تبدیل یک تسک به JSON.
pub fn task_to_json(task: &Task) -> Json {
    Json::obj([
        ("id", Json::from(task.id.get())),
        ("title", Json::from(task.title.as_str())),
        ("priority", Json::from(task.priority.as_str())),
        ("tags", Json::Arr(task.tags.iter().map(|t| Json::from(t.as_str())).collect())),
        ("done", Json::from(task.done)),
        ("created_at", Json::from(task.created_at)),
        (
            "done_at",
            // `Option<u64>` → `Into<Json>` برای Option را در json.rs نوشتیم
            match task.done_at {
                Some(v) => Json::from(v),
                None => Json::Null,
            },
        ),
    ])
}

/// بازخوانی کل عکس لحظه‌ای از JSON.
///
/// # خطاها
/// اگر ساختار داده با انتظار ما نخواند (پیام خطای دقیق می‌دهد).
pub fn snapshot_from_json(value: &Json) -> Result<Snapshot> {
    // بررسی نسخه — «سازگاری رو به عقب» را اینجا مدیریت می‌کنیم.
    let version = value.get_i64("version").unwrap_or(0);
    if version > SCHEMA_VERSION {
        return Err(AppError::BadInput(format!(
            "فایل ذخیره‌سازی نسخهٔ {version} است، ولی این برنامه فقط تا نسخهٔ {SCHEMA_VERSION} را می‌فهمد. \
             برنامه را به‌روز کن."
        )));
    }

    let tasks_json = value
        .get_array("tasks")
        .ok_or_else(|| AppError::BadInput("کلید «tasks» در فایل ذخیره‌سازی پیدا نشد".into()))?;

    // `collect::<Result<Vec<_>>>()` ترفندی است که «وکتور Result» را به
    // «Result وکتور» تبدیل می‌کند — اولی خطا را کوتاه می‌کند.
    let tasks: Vec<Task> = tasks_json
        .iter()
        .map(task_from_json)
        .collect::<Result<Vec<_>>>()?;

    Ok(Snapshot {
        next_id: value.get_i64("next_id").unwrap_or(0) as u64,
        tasks,
    })
}

/// بازخوانی یک تسک از JSON.
///
/// # خطاها
/// اگر فیلدهای اجباری نباشند یا نوعشان نخواند.
pub fn task_from_json(value: &Json) -> Result<Task> {
    let id = value
        .get_i64("id")
        .ok_or_else(|| AppError::BadInput("تسک بدون «id» در فایل".into()))?;

    let title = value
        .get("title")
        .and_then(Json::as_str)
        .ok_or_else(|| AppError::BadInput(format!("تسک #{id} بدون «title» معتبر")))?
        .to_string();

    // اولویت ناشناخته → پیش‌فرض، نه خطا. (تحمل‌پذیری به‌جای سخت‌گیری)
    let priority = value
        .get("priority")
        .and_then(Json::as_str)
        .and_then(|s| s.parse::<Priority>().ok())
        .unwrap_or_default();

    let tags = value
        .get_array("tags")
        .map(|arr| arr.iter().filter_map(|t| t.as_str().map(str::to_string)).collect())
        .unwrap_or_default();

    Ok(Task {
        id: TaskId(id as u64),
        title,
        priority,
        tags,
        done: value.get_bool("done").unwrap_or(false),
        created_at: value.get_i64("created_at").unwrap_or(0) as u64,
        done_at: value.get_i64("done_at").map(|v| v as u64),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
//  کارخانه — ارسال پویا (dynamic dispatch)
// ═══════════════════════════════════════════════════════════════════════════
/// انتخاب انباره بر اساس مسیر داده‌شده.
///
/// خروجی `Box<dyn Store + Send>` است: یک اشاره‌گر هوشمند به «هر چیزی که Store
/// باشد **و بتوان بین تردها فرستادش**». این «تراکنش پویا» است — تصمیم در زمان
/// اجرا گرفته می‌شود، نه کامپایل.
///
/// 🔑 چرا `+ Send` لازم است؟ چون می‌خواهیم `App` را در `Arc<Mutex<..>>` بین
/// تردهای سرور HTTP به اشتراک بگذاریم. بدون این قید، کامپایلر جلوی ما را
/// می‌گیرد: «Box<dyn Store> قابل فرستادن به ترد دیگر نیست» — و همین‌جاست که
/// می‌فهمی صفت‌های خودکار (auto traits) چقدر دقیق کار می‌کنند.
///
/// ⚖️ **مقایسه:**
/// * `Box<dyn Store>` → یک نسخهٔ باینری، کمی هزینهٔ پرش اشاره‌گر (vtable)، انعطاف کامل.
/// * `impl Store` / `T: Store` → صفر هزینه، ولی نوع باید در زمان کامپایل معلوم باشد.
/// هر کدام جای خودش را دارد؛ به `docs/05-traits-generics.md` نگاه کن.
pub fn open_store(path: Option<PathBuf>) -> Box<dyn Store + Send> {
    match path {
        Some(p) => Box::new(FileStore::new(p)),
        None => Box::new(MemoryStore::new()),
    }
}

/// مسیر پیش‌فرض فایل داده.
///
/// ترفند: از متغیر محیطی `AR_DATA` استفاده کن اگر کاربر تنظیمش کرده؛
/// وگرنه `data/tasks.json` در پوشهٔ جاری.
pub fn default_data_path() -> PathBuf {
    match std::env::var_os("AR_DATA") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("data/tasks.json"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  تست‌ها
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Snapshot {
        Snapshot {
            next_id: 3,
            tasks: vec![
                Task::new(TaskId(1), "نوشتن مستندات", Priority::High, vec!["docs".into()]),
                Task::new(TaskId(2), "تست", Priority::Low, vec![]),
            ],
        }
    }

    #[test]
    fn round_trip_through_json() {
        let snap = sample();
        let json = snapshot_to_json(&snap);
        let back = snapshot_from_json(&json).unwrap();
        assert_eq!(back.next_id, 3);
        assert_eq!(back.tasks.len(), 2);
        assert_eq!(back.tasks[0].title, "نوشتن مستندات");
        assert_eq!(back.tasks[0].priority, Priority::High);
        assert_eq!(back.tasks[0].tags, vec!["docs".to_string()]);
        assert_eq!(back.tasks[1].done_at, None);
    }

    #[test]
    fn memory_store_saves_and_loads() {
        let store = MemoryStore::new();
        assert!(!store.is_populated());
        // قبل از ذخیره، بارگذاری باید خالی باشد (مثل فایل ناموجود)
        assert!(store.load().unwrap().tasks.is_empty());
        store.save(&sample()).unwrap();
        assert!(store.is_populated());
        assert_eq!(store.load().unwrap().tasks.len(), 2);
    }

    #[test]
    fn file_store_round_trip_and_atomic_tmp_cleanup() {
        // پوشهٔ موقت اختصاصی این تست — ترفند: نام یکتا با شناسهٔ ترد/زمان
        let dir = std::env::temp_dir().join(format!("ar-store-test-{}", crate::model::now_unix()));
        let path = dir.join("tasks.json");
        let store = FileStore::new(&path);

        // خواندن از فایل ناموجود → خالی، بدون خطا
        assert!(store.load().unwrap().tasks.is_empty());

        store.save(&sample()).unwrap();
        assert!(path.exists(), "فایل اصلی باید ساخته شود");
        assert!(!store.tmp_path().exists(), "فایل موقت باید پاک شده باشد");

        let back = store.load().unwrap();
        assert_eq!(back.tasks.len(), 2);
        assert_eq!(store.describe(), format!("فایل «{}»", path.display()));

        // پاک‌سازی
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn future_schema_is_rejected_with_clear_message() {
        let json = Json::obj([
            ("version", Json::from(999i64)),
            ("tasks", Json::Arr(vec![])),
        ]);
        let err = snapshot_from_json(&json).unwrap_err();
        assert!(err.to_string().contains("نسخهٔ 999"), "{err}");
    }

    #[test]
    fn missing_tasks_key_is_an_error() {
        let json = Json::obj([("version", Json::from(1i64))]);
        assert!(snapshot_from_json(&json).is_err());
    }

    #[test]
    fn unknown_priority_falls_back_to_normal() {
        let json = Json::obj([
            ("id", Json::from(1i64)),
            ("title", Json::from("x")),
            ("priority", Json::from("SUPER-HIGH")),
        ]);
        let task = task_from_json(&json).unwrap();
        assert_eq!(task.priority, Priority::Normal);
    }
}
