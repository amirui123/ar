//! # 🧠 منطق برنامه (Application layer)
//!
//! این ماژول «مغز» پروژه است: [`App`] داده‌ها را در حافظه نگه می‌دارد و
//! روی آن‌ها عملیات انجام می‌دهد ([`App::add`]، [`App::list`]، [`App::stats`] …).
//!
//! جدا نگه داشتن «منطق» از «نمایش» (main.rs) و «ذخیره‌سازی» (store.rs) یک
//! اصل طراحی نرم‌افزار است: تست‌پذیر می‌شود، و می‌توانیم بعداً یک رابط وب یا
//! یک کتابخانه روی همان منطق سوار کنیم — دقیقاً همان کاری که `ar serve` می‌کند.
//!
//! اینجا بیشتر از همه‌جا **ایتراتورها** (iterators) و **بستارها** (closures)
//! می‌بینی؛ قلب سبک کاری Rust.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use crate::errors::{AppError, Result};
use crate::model::{Priority, Task, TaskId, now_unix};
use crate::store::{Snapshot, Store};

// ═══════════════════════════════════════════════════════════════════════════
//  فیلترها و مرتب‌سازی
// ═══════════════════════════════════════════════════════════════════════════
/// کدام تسک‌ها را نشان بدهیم؟
///
/// 🔑 `#[derive(Default)]` روی enum + `#[default]` روی یک واریانت
/// (پایدار از Rust 1.62) → `StatusFilter::default()` همان `All` است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    /// همه
    #[default]
    All,
    /// فقط انجام‌نشده‌ها
    Pending,
    /// فقط انجام‌شده‌ها
    Done,
}

impl StatusFilter {
    /// آیا این تسک از فیلتر رد می‌شود؟
    pub fn accepts(self, task: &Task) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Pending => !task.done,
            StatusFilter::Done => task.done,
        }
    }

    /// پارس از متن کاربر.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "all" | "همه" | "" => Ok(StatusFilter::All),
            "pending" | "todo" | "open" | "باز" | "باقی" => Ok(StatusFilter::Pending),
            "done" | "closed" | "انجام" | "تمام" => Ok(StatusFilter::Done),
            other => Err(AppError::BadInput(format!(
                "وضعیت «{other}» ناشناخته است (all|pending|done)"
            ))),
        }
    }
}

/// معیار مرتب‌سازی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    /// بر اساس شناسه (قدیمی‌ترین اول)
    #[default]
    Id,
    /// بر اساس فوریت (اولویت، سپس سن)
    Urgency,
    /// بر اساس اولویت
    Priority,
    /// بر اساس زمان ساخت (تازه‌ترین اول)
    Newest,
    /// بر اساس زمان ساخت (قدیمی‌ترین اول)
    Oldest,
    /// الفبایی بر اساس عنوان
    Title,
}

impl SortKey {
    /// پارس از متن کاربر.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "id" | "" => Ok(SortKey::Id),
            "urgency" | "فوریت" => Ok(SortKey::Urgency),
            "priority" | "اولویت" => Ok(SortKey::Priority),
            "newest" | "new" | "جدید" => Ok(SortKey::Newest),
            "oldest" | "قدیمی" => Ok(SortKey::Oldest),
            "title" | "عنوان" => Ok(SortKey::Title),
            other => Err(AppError::BadInput(format!(
                "ترتیب «{other}» ناشناخته است (id|urgency|priority|newest|oldest|title)"
            ))),
        }
    }
}

/// مجموعهٔ همهٔ شرط‌های نمایش.
///
/// `Default` برای این است که `Filter::default()` همه‌چیز را بدهد:
/// «همهٔ تسک‌ها، بدون فیلتر، مرتب‌شده بر اساس شناسه».
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// وضعیت
    pub status: StatusFilter,
    /// فقط تسک‌های با این برچسب
    pub tag: Option<String>,
    /// جست‌وجوی متنی در عنوان و برچسب‌ها
    pub search: Option<String>,
    /// معیار مرتب‌سازی
    pub sort: SortKey,
    /// برعکس کردن ترتیب
    pub reverse: bool,
    /// حداکثر تعداد نتیجه (`--limit`)
    pub limit: Option<usize>,
}

impl Filter {
    /// آیا تسک از همهٔ شرط‌ها عبور می‌کند؟
    ///
    /// ⛓️ **زنجیرهٔ ایتراتور:** اینجا `&&` اتصال کوتاه (short-circuit) است،
    /// پس اگر شرط اول رد شد، بقیه حتی *اجرا نمی‌شوند* — یک بهینه‌سازی رایگان.
    pub fn accepts(&self, task: &Task) -> bool {
        self.status.accepts(task)
            && self.tag.as_ref().is_none_or(|tag| task.has_tag(tag))
            && self.search.as_ref().is_none_or(|needle| task.matches(needle))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  آمار
// ═══════════════════════════════════════════════════════════════════════════
/// آمار صفحهٔ `ar stats`.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// تعداد کل
    pub total: usize,
    /// تعداد انجام‌شده
    pub done: usize,
    /// تعداد انجام‌نشده
    pub pending: usize,
    /// درصد انجام‌شده (۰..۱۰۰)
    pub progress_percent: f64,
    /// شمارش به تفکیک اولویت (BTreeMap = همیشه مرتب!)
    pub by_priority: BTreeMap<Priority, usize>,
    /// شمارش به تفکیک برچسب
    pub by_tag: HashMap<String, usize>,
    /// میانگین زمان انجام (ثانیه)
    pub avg_seconds_to_done: Option<u64>,
    /// قدیمی‌ترین تسک باز
    pub oldest_pending: Option<TaskId>,
    /// مجموع «وزن» اولویت‌ها (فوریت کلی)
    pub urgency_total: u32,
}

// ═══════════════════════════════════════════════════════════════════════════
//  App — منطق اصلی
// ═══════════════════════════════════════════════════════════════════════════
/// برنامهٔ درون‌حافظه‌ای با انبارهٔ قابل تعویض.
///
/// نکتهٔ طراحی: `store` را به‌صورت `Box<dyn Store>` نگه می‌داریم و
/// `App` هیچ‌وقت نمی‌داند فایل است یا حافظه. این «وارونگی وابستگی» است.
pub struct App {
    /// انباره (فایل یا حافظه) — `+ Send` تا بتوان App را بین تردها فرستاد
    store: Box<dyn Store + Send>,
    /// تسک‌ها، همیشه مرتب‌شده بر اساس شناسه (تا جست‌وجو با `binary_search` ممکن باشد)
    tasks: Vec<Task>,
    /// شناسهٔ بعدی
    next_id: u64,
    /// آیا از آخرین ذخیره تغییر کرده؟ (برای پرهیز از نوشتن بی‌مورد)
    dirty: bool,
}

impl App {
    /// باز کردن برنامه روی انبارهٔ داده‌شده و بارگذاری داده‌ها.
    ///
    /// # خطاها
    /// اگر خواندن انباره شکست بخورد.
    pub fn open(store: Box<dyn Store + Send>) -> Result<Self> {
        let snap = store.load()?;
        let mut app = App {
            store,
            tasks: snap.tasks,
            next_id: snap.next_id,
            dirty: false,
        };

        // 🛡️ اعتبارسنجی «دادهٔ ورودی»: ممکن است فایل دستی ویرایش شده باشد.
        // مرتب‌سازی + اصلاح next_id تا هیچ‌وقت شناسهٔ تکراری نسازیم.
        app.tasks.sort_by_key(|t| t.id);
        let max_id = app.tasks.last().map(|t| t.id.get()).unwrap_or(0);
        app.next_id = app.next_id.max(max_id + 1);

        Ok(app)
    }

    /// انبارهٔ فعلی (برای پیام‌دادن به کاربر).
    pub fn store_name(&self) -> String {
        self.store.describe()
    }

    /// همهٔ تسک‌ها (فقط خواندنی).
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// آیا تغییرِ ذخیره‌نشده داریم؟
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// افزودن تسک تازه.
    ///
    /// # خطاها
    /// اگر عنوان خالی باشد.
    pub fn add(&mut self, title: &str, priority: Priority, tags: Vec<String>) -> Result<TaskId> {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::BadInput("عنوان تسک نمی‌تواند خالی باشد".into()));
        }

        let id = TaskId(self.next_id);
        self.next_id += 1;
        // ترفند: می‌توانستیم بعداً sort کنیم؛ ولی چون idها صعودی‌اند،
        // `push` در انتها همان ترتیب مرتب را حفظ می‌کند (بدون هزینهٔ sort).
        self.tasks.push(Task::new(id, title, priority, tags));
        self.dirty = true;
        Ok(id)
    }

    /// یافتن تسک با شناسه (تغییرپذیر).
    ///
    /// # خطاها
    /// اگر تسک نباشد.
    pub fn find_mut(&mut self, id: TaskId) -> Result<&mut Task> {
        self.tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(AppError::TaskNotFound(id.get()))
    }

    /// یافتن تسک با شناسه (فقط خواندنی).
    ///
    /// # خطاها
    /// اگر تسک نباشد.
    pub fn get(&self, id: TaskId) -> Result<&Task> {
        self.tasks
            .iter()
            .find(|t| t.id == id)
            .ok_or(AppError::TaskNotFound(id.get()))
    }

    /// انجام‌شده/نشده کردن.
    ///
    /// # خطاها
    /// اگر تسک نباشد.
    pub fn set_done(&mut self, id: TaskId, done: bool) -> Result<bool> {
        let changed = self.find_mut(id)?.set_done(done);
        if changed {
            self.dirty = true;
        }
        Ok(changed)
    }

    /// تغییر اولویت.
    ///
    /// # خطاها
    /// اگر تسک نباشد.
    pub fn set_priority(&mut self, id: TaskId, priority: Priority) -> Result<()> {
        self.find_mut(id)?.priority = priority;
        self.dirty = true;
        Ok(())
    }

    /// حذف تسک و برگرداندن آن.
    ///
    /// # خطاها
    /// اگر تسک نباشد.
    pub fn remove(&mut self, id: TaskId) -> Result<Task> {
        // `position` ایندکس را می‌دهد، `remove` عنصر را بیرون می‌کشد
        // (ترفند: `swap_remove` سریع‌تر است ولی ترتیب را به‌هم می‌ریزد؛
        //  چون ما ترتیب id را می‌خواهیم، `remove` را انتخاب می‌کنیم).
        let idx = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(AppError::TaskNotFound(id.get()))?;
        self.dirty = true;
        Ok(self.tasks.remove(idx))
    }

    /// حذف همهٔ تسک‌های انجام‌شده. تعداد حذف‌شده‌ها را برمی‌گرداند.
    pub fn purge_done(&mut self) -> usize {
        let before = self.tasks.len();
        // `retain` «درجا» فیلتر می‌کند و تخصیص حافظهٔ جدید نمی‌دهد.
        self.tasks.retain(|t| !t.done);
        let removed = before - self.tasks.len();
        if removed > 0 {
            self.dirty = true;
        }
        removed
    }

    /// فهرست تسک‌ها با فیلتر و مرتب‌سازی.
    ///
    /// خروجی `Vec<&Task>` است (قرض، نه کپی) — هیچ تسکی کپی نمی‌شود.
    pub fn list(&self, filter: &Filter) -> Vec<&Task> {
        // ── زنجیرهٔ ایتراتور: فیلتر → مرتب → برعکس → برش ──
        let mut rows: Vec<&Task> = self.tasks.iter().filter(|t| filter.accepts(t)).collect();

        match filter.sort {
            SortKey::Id => rows.sort_by_key(|t| t.id),
            SortKey::Priority => {
                // اولویت بالا اول → پس مقایسه را برعکس می‌کنیم
                rows.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.id.cmp(&b.id)));
            }
            SortKey::Urgency => rows.sort_by(|a, b| a.urgency_cmp(b)),
            SortKey::Newest => rows.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id))),
            SortKey::Oldest => rows.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id))),
            SortKey::Title => rows.sort_by(|a, b| a.title.cmp(&b.title)),
        }

        if filter.reverse {
            rows.reverse();
        }

        // `truncate` با حداکثر مجاز — ترفند امن: اگر n بزرگ‌تر از طول باشد خطا نمی‌دهد
        if let Some(n) = filter.limit {
            rows.truncate(n);
        }

        rows
    }

    /// جست‌وجوی نزدیک‌ترین شناسه‌ها (برای پیام «شاید منظورت #۵ بود؟»).
    ///
    /// این «ترفند UX» است: به‌جای خطای خشک، راهنمایی می‌کنیم.
    pub fn suggest_ids(&self, id: TaskId, count: usize) -> Vec<TaskId> {
        let mut ids: Vec<(u64, TaskId)> = self
            .tasks
            .iter()
            .map(|t| {
                // فاصلهٔ عددی بدون سرریز: همه‌چیز به i128 تا امن باشد
                let delta = (t.id.get() as i128 - id.get() as i128).unsigned_abs() as u64;
                (delta, t.id)
            })
            .collect();
        ids.sort_by_key(|(d, _)| *d);
        ids.into_iter().take(count).map(|(_, id)| id).collect()
    }

    /// محاسبهٔ آمار.
    ///
    /// اینجا از تمام قدرت ایتراتورها استفاده می‌کنیم: `filter`، `fold`،
    /// `filter_map`، `sum`، `min_by_key` و …
    pub fn stats(&self) -> Stats {
        let total = self.tasks.len();
        let done = self.tasks.iter().filter(|t| t.done).count();
        let pending = total - done;

        // تبدیل با `as f64` — مراقب باش: برای عددهای خیلی بزرگ دقت می‌رود.
        let progress_percent = if total == 0 {
            0.0
        } else {
            (done as f64) * 100.0 / (total as f64)
        };

        // شمارش به تفکیک اولویت: اول همه را صفر کن (تا اولویت‌های خالی هم در جدول بیایند)
        let mut by_priority: BTreeMap<Priority, usize> =
            Priority::ALL.iter().map(|p| (*p, 0)).collect();
        for task in &self.tasks {
            // `entry` API: یک بار جست‌وجو، هم برای درج هم افزایش
            *by_priority.entry(task.priority).or_insert(0) += 1;
        }

        // شمارش برچسب‌ها با `flat_map` — یک تسک با ۳ برچسب، سه بار شمرده می‌شود.
        let mut by_tag: HashMap<String, usize> = HashMap::new();
        for tag in self.tasks.iter().flat_map(|t| t.tags.iter()) {
            *by_tag.entry(tag.clone()).or_insert(0) += 1;
        }

        // میانگین زمان انجام: `filter_map` هم فیلتر می‌کند هم تبدیل (Optionها حذف می‌شوند)
        let durations: Vec<u64> = self.tasks.iter().filter_map(Task::time_to_done).collect();
        let avg_seconds_to_done = if durations.is_empty() {
            None
        } else {
            // `sum::<u64>()` ممکن است در تئوری سرریز کند؛ برای امنیت `u128` می‌گیریم
            let sum: u128 = durations.iter().map(|d| *d as u128).sum();
            Some((sum / durations.len() as u128) as u64)
        };

        let oldest_pending = self
            .tasks
            .iter()
            .filter(|t| !t.done)
            .min_by_key(|t| t.created_at)
            .map(|t| t.id);

        let urgency_total = self
            .tasks
            .iter()
            .filter(|t| !t.done)
            .map(|t| t.priority.weight())
            .sum();

        Stats {
            total,
            done,
            pending,
            progress_percent,
            by_priority,
            by_tag,
            avg_seconds_to_done,
            oldest_pending,
            urgency_total,
        }
    }

    /// ذخیره اگر تغییر کرده باشد. خروجی: آیا نوشتیم؟
    ///
    /// # خطاها
    /// اگر نوشتن شکست بخورد.
    pub fn save(&mut self) -> Result<bool> {
        if !self.dirty {
            return Ok(false);
        }
        let snap = Snapshot { next_id: self.next_id, tasks: self.tasks.clone() };
        self.store.save(&snap)?;
        self.dirty = false;
        Ok(true)
    }

    /// افزودن چند تسک نمونه (برای `ar seed`).
    pub fn seed(&mut self) -> Result<usize> {
        let samples: [(&str, Priority, &[&str]); 5] = [
            ("یاد گرفتن ownership در Rust", Priority::Urgent, &["rust", "یادگیری"]),
            ("نوشتن سرور HTTP بدون وابستگی", Priority::High, &["rust", "http"]),
            ("مرور کدهای هم‌تیمی", Priority::Normal, &["کار"]),
            ("خرید نان", Priority::Low, &["خانه"]),
            ("پیاده‌سازی mini async runtime", Priority::High, &["rust", "async"]),
        ];
        let mut n = 0;
        for (title, prio, tags) in samples {
            self.add(title, prio, tags.iter().map(|s| s.to_string()).collect())?;
            n += 1;
        }
        Ok(n)
    }
}

impl std::fmt::Debug for App {
    /// `Debug` دستی: چون `Box<dyn Store>` خودش `Debug` نیست و `#[derive]` کار نمی‌کند.
    /// 📌 **درس:** گاهی لازم است صفت‌هایی را دستی پیاده کنی؛ در آن صورت *قرارداد*
    /// صفت را رعایت کن (Debug باید ساختاری و خوانا باشد، نه چیز عجیب).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("store", &self.store_name())
            .field("tasks", &self.tasks.len())
            .field("next_id", &self.next_id)
            .field("dirty", &self.dirty)
            .finish()
    }
}

/// مقایسهٔ دو تسک بر اساس زمان ساخت (کمکی و عمومی، برای استفادهٔ بیرونی).
pub fn by_created(a: &Task, b: &Task) -> Ordering {
    a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id))
}

/// زمان «الان» — بازصادرشده برای راحتی مصرف‌کننده‌های این ماژول.
pub fn now() -> u64 {
    now_unix()
}

// ═══════════════════════════════════════════════════════════════════════════
//  تست‌ها
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    /// کمکی: برنامهٔ تازه روی حافظه
    fn app() -> App {
        App::open(Box::new(MemoryStore::new())).unwrap()
    }

    #[test]
    fn add_and_list() {
        let mut a = app();
        let id1 = a.add("تست یک", Priority::Normal, vec![]).unwrap();
        let id2 = a.add("تست دو", Priority::Urgent, vec!["x".into()]).unwrap();
        assert_eq!(id1, TaskId(1));
        assert_eq!(id2, TaskId(2));
        assert_eq!(a.list(&Filter::default()).len(), 2);
        assert!(a.is_dirty());
    }

    #[test]
    fn empty_title_rejected() {
        let mut a = app();
        assert!(a.add("   ", Priority::Low, vec![]).is_err());
    }

    #[test]
    fn done_and_undo() {
        let mut a = app();
        let id = a.add("کار", Priority::Normal, vec![]).unwrap();
        assert!(a.set_done(id, true).unwrap());
        assert!(!a.set_done(id, true).unwrap()); // بار دوم تغییری نیست
        assert!(a.get(id).unwrap().done);
        assert!(a.set_done(id, false).unwrap());
        assert!(!a.get(id).unwrap().done);
    }

    #[test]
    fn filtering_works() {
        let mut a = app();
        let x = a.add("خرید نان", Priority::Low, vec!["خانه".into()]).unwrap();
        a.add("کد بزن", Priority::High, vec!["کار".into()]).unwrap();
        a.set_done(x, true).unwrap();

        let f_pending = Filter { status: StatusFilter::Pending, ..Default::default() };
        assert_eq!(a.list(&f_pending).len(), 1);

        let f_tag = Filter { tag: Some("خانه".into()), ..Default::default() };
        assert_eq!(a.list(&f_tag).len(), 1);

        let f_search = Filter { search: Some("کد".into()), ..Default::default() };
        assert_eq!(a.list(&f_search).len(), 1);

        let f_limit = Filter { limit: Some(1), ..Default::default() };
        assert_eq!(a.list(&f_limit).len(), 1);
    }

    #[test]
    fn sorting_variants() {
        let mut a = app();
        let low = a.add("کم‌اهمیت", Priority::Low, vec![]).unwrap();
        let high = a.add("مهم", Priority::High, vec![]).unwrap();

        let f = Filter { sort: SortKey::Priority, ..Default::default() };
        assert_eq!(a.list(&f)[0].id, high);

        let f = Filter { sort: SortKey::Priority, reverse: true, ..Default::default() };
        assert_eq!(a.list(&f)[0].id, low);
    }

    #[test]
    fn remove_and_suggest() {
        let mut a = app();
        let id = a.add("الف", Priority::Normal, vec![]).unwrap();
        a.add("ب", Priority::Normal, vec![]).unwrap();
        assert_eq!(a.remove(id).unwrap().id, id);
        assert!(a.remove(id).is_err());
        // پیشنهاد نزدیک‌ترین شناسه‌ها
        let s = a.suggest_ids(TaskId(1), 2);
        assert_eq!(s, vec![TaskId(2)]);
    }

    #[test]
    fn purge_removes_done_only() {
        let mut a = app();
        let x = a.add("x", Priority::Normal, vec![]).unwrap();
        a.add("y", Priority::Normal, vec![]).unwrap();
        a.set_done(x, true).unwrap();
        assert_eq!(a.purge_done(), 1);
        assert_eq!(a.tasks().len(), 1);
    }

    #[test]
    fn stats_are_correct() {
        let mut a = app();
        let x = a.add("الف", Priority::High, vec!["t".into()]).unwrap();
        a.add("ب", Priority::Low, vec!["t".into(), "u".into()]).unwrap();
        a.set_done(x, true).unwrap();

        let s = a.stats();
        assert_eq!(s.total, 2);
        assert_eq!(s.done, 1);
        assert_eq!(s.pending, 1);
        assert!((s.progress_percent - 50.0).abs() < f64::EPSILON);
        assert_eq!(s.by_priority[&Priority::High], 1);
        assert_eq!(s.by_tag["t"], 2);
        assert_eq!(s.by_tag["u"], 1);
        assert!(s.avg_seconds_to_done.is_some());
        assert!(s.oldest_pending.is_some());
        assert_eq!(s.urgency_total, Priority::Low.weight());
    }

    #[test]
    fn invalid_snapshot_is_repaired_on_open() {
        // شبیه‌سازی فایل دستکاری‌شده: next_id کوچک‌تر از شناسه‌های موجود
        let snap = Snapshot {
            next_id: 1,
            tasks: vec![
                Task::new(TaskId(9), "نهم", Priority::Normal, vec![]),
                Task::new(TaskId(3), "سوم", Priority::Normal, vec![]),
            ],
        };
        let store = MemoryStore::new();
        store.save(&snap).unwrap();
        let mut a = App::open(Box::new(store)).unwrap();
        assert_eq!(a.tasks()[0].id, TaskId(3), "باید مرتب شود");
        let new_id = a.add("تازه", Priority::Normal, vec![]).unwrap();
        assert_eq!(new_id, TaskId(10), "نباید شناسهٔ تکراری بسازد");
    }

    #[test]
    fn save_is_idempotent_and_only_when_dirty() {
        let mut a = app();
        assert!(!a.save().unwrap(), "چیزی تغییر نکرده → نباید بنویسد");
        a.add("x", Priority::Normal, vec![]).unwrap();
        assert!(a.save().unwrap());
        assert!(!a.save().unwrap(), "بار دوم دیگر کثیف نیست");
        assert!(!a.is_dirty());
    }
}
