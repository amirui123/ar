//! # گزارش از فایل داده (نمونهٔ واقعی‌تر)
//!
//! اجرا:
//! ```bash
//! cargo run --example report                 # از فایل پیش‌فرض data/tasks.json
//! cargo run --example report -- /tmp/x.json  # از مسیر دلخواه
//! ```
//!
//! اگر فایل نباشد، به‌جای خطا یک مجموعهٔ نمونه در حافظه ساخته می‌شود —
//! الگوی «کار کند حتی وقتی داده نیست».

use ar::app::{App, Filter, SortKey, StatusFilter};
use ar::model::{Priority, humanize_age, now_unix};

fn main() -> ar::Result<()> {
    // ── ۱) انتخاب انباره بر اساس آرگومان خط فرمان ──
    // `args().nth(1)` = اولین آرگومان بعد از نام برنامه.
    // مسیر متنی را به `PathBuf` تبدیل می‌کنیم (`open_store` نوع مسیر می‌خواهد).
    let path = std::env::args().nth(1).map(std::path::PathBuf::from);
    let store = ar::store::open_store(path);

    let mut app = App::open(store)?;

    // ── ۲) اگر داده‌ای نبود، نمونه بساز ──
    if app.tasks().is_empty() {
        app.seed()?;
        println!("(فایل داده خالی بود؛ یک مجموعهٔ نمونه در حافظه ساخته شد)\n");
    }

    let s = app.stats();

    // ── ۳) سرصفحه ──
    println!("════════ گزارش تسک‌ها ════════");
    println!("منبع           : {}", app.store_name());
    println!("کل             : {}", s.total);
    println!("انجام‌شده        : {} ({}٪)", s.done, s.progress_percent.round());
    println!("باقی‌مانده       : {}", s.pending);
    println!("مجموع فوریت     : {}", s.urgency_total);

    // ── ۴) توزیع اولویت‌ها ──
    // `by_priority` یک `BTreeMap` است → همیشه مرتب (کم → فوری).
    println!("\n── به تفکیک اولویت ──");
    for p in Priority::ALL {
        let n = s.by_priority.get(&p).copied().unwrap_or(0);
        if n == 0 {
            continue;
        }
        // نوارِ سادهٔ متنی: هر خانه = ۵٪
        let bar = "█".repeat((n * 20 / s.total.max(1)).max(1));
        println!("  {:>6} │ {:<3} {}", p.label(), n, bar);
    }

    // ── ۵) توزیع برچسب‌ها (مرتب‌شده، چون HashMap ترتیب ندارد) ──
    if !s.by_tag.is_empty() {
        let mut tags: Vec<(&String, &usize)> = s.by_tag.iter().collect();
        // مرتب‌سازی نزولی بر اساس تعداد، و در تساوی، الفبایی.
        tags.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        println!("\n── پرتکرارترین برچسب‌ها ──");
        for (tag, n) in tags.iter().take(5) {
            println!("  #{tag:<12} {n}");
        }
    }

    // ── ۶) زمان ──
    if let Some(avg) = s.avg_seconds_to_done {
        println!("\nمیانگین زمان انجام: {} ثانیه", avg);
    }
    if let Some(oldest) = s.oldest_pending {
        // `find` روی ایتراتور تا اولین تطبیق (نه `filter().next()`).
        if let Some(t) = app.tasks().iter().find(|t| t.id == oldest) {
            let age = now_unix().saturating_sub(t.created_at);
            println!("قدیمی‌ترین کار باز: {} ({})", t.id, humanize_age(age));
        }
    }

    // ── ۷) سه تسک فوری بعدی ──
    let next: Vec<_> = app
        .list(&Filter {
            status: StatusFilter::Pending,   // فقط کارهای باز
            sort: SortKey::Urgency,          // فوری‌ترین و قدیمی‌ترین اول
            limit: Some(3),                  // ترفند: `limit` در فیلتر، نه در چاپ
            ..Filter::default()
        })
        .into_iter()
        .map(|t| format!("{} {}", t.id, t.title))   // از `&Task` به رشته
        .collect();

    println!("\n🎯 سه کار بعدی:");
    for line in &next {
        println!("  • {line}");
    }

    // 💡 بونوس: برای دیدن دادهٔ خام، `ar::store::snapshot_to_json` را ببین.
    Ok(())
}
