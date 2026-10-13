//! # تست‌های یکپارچه (Integration Tests)
//!
//! تفاوت این فایل با تست‌های واحد داخل `src/`:
//!
//! * تست‌های واحد (`#[cfg(test)] mod tests`) **داخل** ماژول‌ها هستند و به
//!   آیتم‌های خصوصی (`pub(crate)`) هم دسترسی دارند.
//! * تست‌های یکپارچه در پوشهٔ جداگانهٔ `tests/` هستند و فقط از **API عمومی**
//!   کریت (`ar::…`) استفاده می‌کنند — یعنی دقیقاً همان چیزی که یک کاربر بیرونی
//!   می‌بیند.
//!
//! 💡 ترفند آموزشی: چون تست یکپارچه فقط از `ar::` استفاده می‌کند، اگر
//! چیزی را از `lib.rs` حذف یا خصوصی کنی، **همین‌جا** خطا می‌گیری و
//! می‌فهمی که API عمومی‌ات شکسته است.

use ar::app::{App, Filter, SortKey, StatusFilter};
use ar::model::{Priority, TaskId};
use ar::store::{FileStore, MemoryStore, Snapshot, snapshot_from_json, snapshot_to_json};

// ═══════════════════════════════════════════════════════════════════════════
//  کمک‌کننده‌ها
// ═══════════════════════════════════════════════════════════════════════════

/// یک برنامهٔ تازه روی انبارهٔ «درون‌حافظه» می‌سازد.
///
/// چرا حافظه و نه فایل؟ چون تست باید **سریع، مستقل و تکرارپذیر** باشد؛
/// هیچ تستی نباید به فایل روی دیسک وابسته باشد (مگر تستی که موضوعش خودِ
/// ذخیره‌سازی باشد — مثل `file_store_persists_across_app_instances`).
fn app() -> App {
    App::open(Box::new(MemoryStore::new())).expect("انبارهٔ حافظه‌ای هرگز شکست نمی‌خورد")
}

/// یک فیلتر «همه‌چیز» می‌سازد که بعداً فیلدهایش را عوض می‌کنیم.
///
/// `..Filter::default()` (به‌روزرسانی ساختاری) باعث می‌شود لازم نباشد هر
/// شش فیلد را دستی بنویسیم؛ اگر فردا فیلد جدیدی اضافه شود، کد تست نمی‌شکند.
fn filter() -> Filter {
    Filter::default()
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۱) ساخت و شناسه‌ها
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn add_assigns_increasing_ids() {
    let mut app = app();

    // `add` شناسهٔ بعدی را خودش تخصیص می‌دهد و آن را برمی‌گرداند.
    let a = app.add("نوشتن مستندات", Priority::Normal, vec!["docs".into()]).unwrap();
    let b = app.add("رفع باگ پارسر", Priority::Urgent, vec!["cli".into(), "bug".into()]).unwrap();

    // 🎯 الگوی آموزشی: `TaskId` یک newtype است؛ برای مقایسهٔ عددی، صریح
    // `get()` می‌زنیم تا خواننده بفهمد عددِ خام را می‌خواهیم (نه هویت نوع).
    assert_eq!(a.get(), 1);
    assert_eq!(b.get(), 2);
    assert_eq!(app.tasks().len(), 2);

    // ساخت تسک تازه همیشه «کثیف» می‌کند = نیاز به `save`.
    assert!(app.is_dirty(), "بعد از add باید ذخیره‌نشده باشد");
}

#[test]
fn seed_creates_a_useful_demo_set() {
    let mut app = app();
    let n = app.seed().expect("دادهٔ نمونه باید ساخته شود");
    assert!(n >= 5, "انتظار داریم چند تسک نمونه ساخته شود، گرفتیم {n}");
    assert_eq!(app.tasks().len(), n);
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۲) وضعیت انجام و آمار
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn set_done_reports_change_and_updates_stats() {
    let mut app = app();
    let id = app.add("کار اول", Priority::Normal, vec![]).unwrap();
    app.add("کار دوم", Priority::Normal, vec![]).unwrap();

    // `set_done(true)` مقدار `true` می‌دهد = «وضعیت عوض شد».
    assert!(app.set_done(id, true).unwrap());
    // بار دوم چیزی عوض نمی‌شود → `false` (کد ما کار بی‌فایده نمی‌کند).
    assert!(!app.set_done(id, true).unwrap());

    let s = app.stats();
    assert_eq!(s.total, 2);
    assert_eq!(s.done, 1);
    assert_eq!(s.pending, 1);
    // 🎯 مقایسهٔ اعشاری: هرگز `assert_eq!` مستقیم روی f64 نزن، همیشه با دلتا.
    assert!((s.progress_percent - 50.0).abs() < f64::EPSILON, "درصد: {}", s.progress_percent);

    // برگرداندن به حالت «باز» باید `done_at` را هم پاک کند.
    assert!(app.set_done(id, false).unwrap());
    assert_eq!(app.stats().done, 0);
    assert!(app.get(id).unwrap().done_at.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۳) فیلترها، جست‌وجو و مرتب‌سازی
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_by_status_tag_and_search() {
    let mut app = app();
    let docs = app.add("نوشتن مستندات", Priority::Normal, vec!["docs".into()]).unwrap();
    app.add("رفع باگ شبکه", Priority::Urgent, vec!["net".into()]).unwrap();
    app.set_done(docs, true).unwrap();

    // ۱) فیلتر وضعیت
    let done = app.list(&Filter { status: StatusFilter::Done, ..filter() });
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].id, docs);

    // ۲) فیلتر برچسب
    let net = app.list(&Filter { tag: Some("net".into()), ..filter() });
    assert_eq!(net.len(), 1);

    // ۳) جست‌وجوی متنی (بی‌توجه به بزرگی/کوچکی حروف)
    let found = app.list(&Filter { search: Some("مستندات".into()), ..filter() });
    assert_eq!(found.len(), 1);

    // ۴) ترکیب شرط‌ها = «و» منطقی؛ چیزی که نه done است نه docs → خالی
    let contradictory = app.list(&Filter {
        status: StatusFilter::Pending,
        tag: Some("docs".into()),
        ..filter()
    });
    assert!(contradictory.is_empty());
}

#[test]
fn sorting_and_limit_and_reverse() {
    let mut app = app();
    let low = app.add("کم‌اهمیت", Priority::Low, vec![]).unwrap();
    let urgent = app.add("فوری", Priority::Urgent, vec![]).unwrap();
    app.add("معمولی", Priority::Normal, vec![]).unwrap();

    // مرتب‌سازی بر اساس اولویت: فوری اول، کم‌اهمیت آخر.
    let by_priority = app.list(&Filter { sort: SortKey::Priority, ..filter() });
    assert_eq!(by_priority[0].id, urgent);
    assert_eq!(by_priority[2].id, low);

    // `reverse` ترتیب را برمی‌گرداند (بدون تغییر جای منطق مرتب‌سازی).
    let reversed = app.list(&Filter { sort: SortKey::Priority, reverse: true, ..filter() });
    assert_eq!(reversed[0].id, low);

    // `limit` مثل `head` عمل می‌کند و اگر بزرگ‌تر از طول باشد خطا نمی‌دهد.
    assert_eq!(app.list(&Filter { limit: Some(2), ..filter() }).len(), 2);
    assert_eq!(app.list(&Filter { limit: Some(99), ..filter() }).len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۴) حذف و پاک‌سازی
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn remove_returns_task_and_purge_removes_done_ones() {
    let mut app = app();
    let a = app.add("الف", Priority::Normal, vec![]).unwrap();
    let b = app.add("ب", Priority::Normal, vec![]).unwrap();
    app.add("پ", Priority::Normal, vec![]).unwrap();
    app.set_done(b, true).unwrap();

    // `remove` تسک حذف‌شده را **برمی‌گرداند** (مثل `Vec::remove`) — برای
    // نمایش «این را حذف کردم» یا امکان بازگردانی مفید است.
    let removed = app.remove(a).unwrap();
    assert_eq!(removed.title, "الف");
    assert_eq!(app.tasks().len(), 2);

    // `purge_done` تعداد حذف‌شده‌ها را می‌دهد، نه فهرست آن‌ها.
    assert_eq!(app.purge_done(), 1);
    assert_eq!(app.tasks().len(), 1);
    // حذف دوباره‌ای که چیزی برای حذف ندارد = صفر (نه خطا).
    assert_eq!(app.purge_done(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۵) خطاها و راهنمایی کاربر
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn missing_id_gives_error_and_suggestions() {
    let mut app = app();
    app.add("تسک یک", Priority::Normal, vec![]).unwrap();
    app.add("تسک دو", Priority::Normal, vec![]).unwrap();
    app.add("تسک سه", Priority::Normal, vec![]).unwrap();

    // 🎯 الگو: خطای مورد انتظار را با `matches!` چک می‌کنیم؛ این
    // «تست کردن واریانت خطا» است، نه فقط «خطا داد یا نه».
    let err = app.get(TaskId(99)).unwrap_err();
    assert!(
        matches!(err, ar::AppError::TaskNotFound(99)),
        "انتظار TaskNotFound(99) داشتیم، گرفتیم: {err:?}"
    );

    // پیام نمایشی باید فارسی و حاوی شماره باشد (تجربهٔ کاربری).
    let msg = err.to_string();
    assert!(msg.contains("99"), "پیام خطا باید شماره را داشته باشد: {msg}");

    // `suggest_ids` نزدیک‌ترین شناسه‌ها را می‌دهد: 99 → 3، 2، 1
    let near = app.suggest_ids(TaskId(99), 3);
    assert_eq!(near, vec![TaskId(3), TaskId(2), TaskId(1)]);
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۶) ذخیره‌سازی روی فایل (رفت‌وبرگشت واقعی)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn file_store_persists_across_app_instances() {
    // مسیر موقت یکتا: شناسهٔ پروسه + زمان، تا تست‌های موازی به‌هم نخورند.
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("ar-it-{}-{unique}.json", std::process::id()));

    // ── نمونهٔ اول: می‌نویسد ──
    {
        let mut app = App::open(Box::new(FileStore::new(&path))).unwrap();
        let id = app.add("ماندگار", Priority::High, vec!["persist".into()]).unwrap();
        app.set_done(id, true).unwrap();
        assert!(app.save().unwrap(), "ذخیره باید تغییری نوشته باشد");
        // `save` بعد از نوشتن، دوباره «تمیز» می‌شود.
        assert!(!app.is_dirty());
        // ذخیرهٔ دوم بدون تغییر باید `false` بدهد (بدون نوشتن دوباره).
        assert!(!app.save().unwrap());
    }

    // ── نمونهٔ دوم: بازمی‌خواند ──
    {
        let app = App::open(Box::new(FileStore::new(&path))).unwrap();
        assert_eq!(app.tasks().len(), 1);
        let t = &app.tasks()[0];
        assert_eq!(t.title, "ماندگار");
        assert_eq!(t.priority, Priority::High);
        assert!(t.done);
        assert!(t.has_tag("persist"));
    }

    // ── تمیزکاری: تست نباید زباله بگذارد ──
    let _ = std::fs::remove_file(&path);
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۷) JSON: رفت‌وبرگشت بدون از دست رفتن داده
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn json_export_round_trips() {
    let mut app = app();
    let id = app.add("کار «گیومه‌دار»", Priority::Urgent, vec!["a".into(), "b".into()]).unwrap();
    app.set_done(id, true).unwrap();

    // ساخت Snapshot از وضعیت فعلی برنامه (چیزی که `export` چاپ می‌کند).
    let snap = Snapshot {
        next_id: app.tasks().iter().map(|t| t.id.get()).max().unwrap_or(0) + 1,
        tasks: app.tasks().to_vec(),
    };

    // ۱) → Json → متن زیبا
    let json = snapshot_to_json(&snap);
    let text = json.to_pretty(2);

    // ۲) متن → Json → Snapshot (با پارسر دست‌ساز خودمان!)
    let parsed = ar::json::parse(&text).expect("خروجی خودمان باید قابل پارس باشد");
    let back = snapshot_from_json(&parsed).expect("شکل داده باید برگردد");

    // 🎯 ترفند تست: مقایسهٔ «دور کامل» — اگر هر فیلدی در نوشتن یا خواندن
    // گم شود، همان‌جا لو می‌رود. برای همین بهترین تست برای هر سریالایزر است.
    assert_eq!(back.tasks.len(), 1);
    let t = &back.tasks[0];
    assert_eq!(t.id.get(), 1);
    assert_eq!(t.title, "کار «گیومه‌دار»", "رشته‌های فارسی با گیومه باید سالم بمانند");
    assert_eq!(t.priority, Priority::Urgent);
    assert_eq!(t.tags, vec!["a".to_string(), "b".to_string()]);
    assert!(t.done);
}
