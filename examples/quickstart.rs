//! # کوتاه‌ترین مسیر استفاده از کتابخانهٔ `ar`
//!
//! اجرا:
//! ```bash
//! cargo run --example quickstart
//! ```
//!
//! 💡 نکتهٔ آموزشی: این فایل **بیرون** از `src/` است، پس فقط به API عمومی
//! (`ar::…`) دسترسی دارد — دقیقاً مثل کسی که این کریت را به‌عنوان وابستگی
//! اضافه کرده باشد.

// `ar::Result` نوع نتیجهٔ خودِ پروژه است (alias روی `Result<T, AppError>`).
fn main() -> ar::Result<()> {
    // ۱) انتخاب انباره: `MemoryStore` هیچ‌جا چیزی نمی‌نویسد (مناسب نمونه/تست).
    //    🎯 اینجا «وارونگی وابستگی» را می‌بینیم: `App` نمی‌داند انباره چیست.
    let mut app = App::open(Box::new(MemoryStore::new()))?;

    // ۲) افزودن تسک — `Priority` یک enum است، پس مقدار نامعتبر **غیرممکن** است.
    let docs = app.add("خواندن مستندات پروژه", Priority::High, vec!["study".into()])?;
    let code = app.add("نوشتن یک ماژول تازه", Priority::Normal, vec!["code".into()])?;

    // ۳) علامت‌زدن یکی از تسک‌ها (🥇 `?` خطا را به بالا پخش می‌کند).
    app.set_done(docs, true)?;

    // ۴) خواندن فهرست. `Filter::default()` = همه، مرتب بر اساس شناسه.
    println!("📋 فهرست تسک‌ها:");
    for task in app.list(&Filter::default()) {
        // `Display for Task` خودش «✔»/«☐» را می‌گذارد.
        println!("   {task}  ({})", task.priority.label());
    }

    // ۵) آمار — یک ساختار آماده، نه محاسبات دستی.
    let s = app.stats();
    println!(
        "\n📊 پیشرفت: {}/{} تسک ({}٪) — {} مورد باقی مانده",
        s.done, s.total, s.progress_percent.round(), s.pending
    );
    println!("🔥 مجموع فوریت: {}", s.urgency_total);
    println!("🔑 شناسهٔ تسک کد: {code}");

    Ok(())
}

// ── ایمپورت‌ها (پایین فایل عمداً، تا متن درس تمیز بماند) ───────────────────
use ar::app::{App, Filter};
use ar::model::Priority;
use ar::store::MemoryStore;
