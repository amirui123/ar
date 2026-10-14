//! # تولید گزارش HTML ایستا — «HTML با زبان Rust»
//!
//! این ماژول یک صفحهٔ **کاملاً مستقل** می‌سازد: داده‌ها داخلش حک شده‌اند
//! (inline)، هیچ `fetch`، هیچ سرور و هیچ فایل جانبی لازم نیست. فایل را
//! دوبار کلیک کنی، باز می‌شود.
//!
//! ## چرا دو نسخهٔ وب داریم؟
//!
//! | | `web.rs` (سرور زنده) | `report.rs` (این ماژول) |
//! |---|---|---|
//! | نیاز به اجرای برنامه | ✅ بله | ❌ نه (فایل ایستا) |
//! | ساخت/تغییر تسک | ✅ | ❌ فقط خواندن |
//! | اشتراک‌گذاری/ایمیل/آرشیو | ⚠️ سخت | ✅ عالی |
//! | کارکرد آفلاین | روی خود کامپیوتر | هر جا |
//!
//! ## تصمیم طراحی مهم: «افزایش تدریجی» (Progressive Enhancement)
//!
//! همهٔ ردیف‌ها **در سمت Rust** به HTML تبدیل می‌شوند؛ جاوااسکریپت فقط
//! `hide/show` می‌کند. یعنی اگر JS خاموش باشد هم صفحه کامل و خواناست،
//! و اگر روشن باشد فیلتر/جست‌وجو هم کار می‌کند. این برعکس روش رایجی است
//! که صفحه بدون JS کاملاً خالی می‌ماند.
//!
//! ## 🔐 نکتهٔ امنیتی حیاتی: فرار دادن متن (HTML escaping)
//!
//! عنوان تسک از کاربر می‌آید. اگر آن را خام داخل HTML بگذاریم:
//!
//! ```text
//! عنوان:  <script>alert('XSS')</script>
//! خروجی:   <li><script>alert('XSS')</script></li>   ← 💀 اجرا می‌شود!
//! ```
//!
//! پس **هر** متنی که از داده می‌آید از [`escape_html`] عبور می‌کند. این
//! همان کاری است که موتورهای قالب (Askama/Tera/Maud) هم خودکار انجام می‌دهند.

use std::fmt::Write as _;
use std::path::Path;

use crate::app::{App, Stats};
use crate::errors::{AppError, Result};
use crate::model::{Priority, Task, humanize_age, now_unix};

// ═══════════════════════════════════════════════════════════════════════════
//  ابزارهای کوچک: فرار متن و تاریخ
// ═══════════════════════════════════════════════════════════════════════════

/// متن را برای درج امن در HTML آماده می‌کند.
///
/// پنج کاراکتری که HTML برایشان معنی خاص دارد:
///
/// | کاراکتر | جایگزین | چرا |
/// |---------|---------|-----|
/// | `&` | `&amp;` | شروع موجودیت (باید **اول** جایگزین شود) |
/// | `<` | `&lt;` | شروع تگ |
/// | `>` | `&gt;` | پایان تگ |
/// | `"` | `&quot;` | بستن مقدار اتریبیوت |
/// | `'` | `&#39;` | اتریبیوت با تک‌کوتیشن |
///
/// 🔑 ترتیب مهم است: اگر اول `<` را عوض کنی و بعد `&`، نتیجهٔ `&lt;`
/// دوباره تبدیل می‌شود و متن خراب می‌شود. `&` همیشه اول.
pub fn escape_html(input: &str) -> String {
    // `with_capacity` = یک تخصیص، بدون رشد پیاپی (بخش ۱۴: کارایی)
    let mut out = String::with_capacity(input.len() + input.len() / 8);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // بقیهٔ کاراکترها (شامل فارسی!) بدون تغییر رد می‌شوند
            other => out.push(other),
        }
    }
    out
}

/// تبدیل «روزهای از مبدأ یونیکس» به تاریخ میلادی: `(سال، ماه، روز)`.
///
/// 🎯 **چرا لازم است؟** `std` هیچ نوع «تاریخ» ندارد (نه `Date` مثل JS، نه
/// `datetime` مثل پایتون) — چون تقویم، منطقهٔ زمانی، ثانیهٔ کبیسه و … مسائل
/// پیچیده‌ای هستند و Rust تصمیم گرفت آن‌ها را به crateها بسپارد (`chrono`,
/// `time`, `jiff`). ولی برای یک گزارش ساده، خودِ الگوریتم فقط چند خط است.
///
/// این الگوریتم معروف «civil_from_days» است (هاوارد هینانت): روزهای مبدأ
/// گریگوری را به سال/ماه/روز تبدیل می‌کند، بدون هیچ حلقه‌ای و بدون جدول.
pub fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    // جابه‌جایی مبدأ به ۱ مارس سال ۰ (تا سال کبیسه انتهای سال بیفتد و
    // محاسبه ساده شود) — عدد جادویی ۷۱۹۴۶۸ در الگوریتم اصلی.
    let z = days_since_epoch + 719_468;

    // «دورهٔ» ۴۰۰ سالهٔ گریگوری = ۱۴۶۰۹۷ روز
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64; // [0, 146096]

    // سال درون دوره: با کم‌کردن روزهای کبیسه، به تقویم ۳۶۵ روزه می‌رسیم
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;

    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    // ۱۵۳ = طول ۵ ماه متوالی ۳۱ روزه در تقویم جابه‌جاشده
    let mp = (5 * day_of_year + 2) / 153; // [0, 11] — ماه با شروع از مارس
    let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]

    // ژانویه و فوریه در «سال جابه‌جاشده» به سال بعد تعلق دارند
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// تبدیل ثانیه‌های یونیکس به تاریخ و ساعت UTC: `(سال، ماه، روز، ساعت، دقیقه، ثانیه)`.
pub fn datetime_utc(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, mo, d) = civil_from_days(days);
    (y, mo, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// قالب کوتاه تاریخ برای نمایش: `۲۰۲۶-۱۰-۱۴ ۰۱:۴۲` (رقم لاتین، خوانا).
pub fn format_datetime(secs: u64) -> String {
    let (y, mo, d, h, mi, _s) = datetime_utc(secs);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}")
}

// ═══════════════════════════════════════════════════════════════════════════
//  ساخت گزارش
// ═══════════════════════════════════════════════════════════════════════════

/// کلاس CSS مربوط به هر اولویت (رنگ پیل).
fn priority_class(p: Priority) -> &'static str {
    match p {
        Priority::Urgent => "p-urgent",
        Priority::High => "p-high",
        Priority::Normal => "p-normal",
        Priority::Low => "p-low",
    }
}

/// یک ردیف تسک را به HTML تبدیل می‌کند.
///
/// 🌟 **ترفند `data-*`:** به‌جای آنکه جاوااسکریپت برای فیلترکردن مجبور شود
/// به متن دیده‌شده نگاه کند (شکننده)، همان‌جا داده‌های ساختاری را در
/// اتریبیوت‌های `data-status` / `data-prio` / `data-text` می‌گذاریم. JS فقط
/// `dataset` را می‌خواند. این همان کاری است که فریم‌ورک‌ها با «props» می‌کنند.
fn task_row(task: &Task, age_now: u64) -> String {
    let mark = if task.done { "✔" } else { "☐" };
    let status = if task.done { "done" } else { "pending" };
    let age = humanize_age(age_now.saturating_sub(task.created_at));

    let mut row = String::with_capacity(256);
    row.push_str(&format!(
        r#"<li class="task{done_cls}" data-status="{status}" data-prio="{prio}" data-text="{needle}">"#,
        done_cls = if task.done { " done" } else { "" },
        status = status,
        prio = task.priority.as_str(),
        // برای جست‌وجو: عنوان + برچسب‌ها، کوچک‌شده و فرارداده‌شده
        needle = escape_html(
            &format!("{} {}", task.title, task.tags.join(" ")).to_lowercase()
        ),
    ));

    // شناسه و عنوان
    row.push_str(&format!(
        r#"<span class="id">{mark} {id}</span><span class="title">{title}</span>"#,
        mark = mark,
        id = task.id, // Display خودش «#12» می‌گذارد
        title = escape_html(&task.title),
    ));

    // پیل اولویت
    row.push_str(&format!(
        r#"<span class="pill {cls}">{label}</span>"#,
        cls = priority_class(task.priority),
        label = task.priority.label(),
    ));

    // برچسب‌ها
    if !task.tags.is_empty() {
        row.push_str(r#"<span class="tags">"#);
        for tag in &task.tags {
            row.push_str(&format!(r#"<span class="tag">#{}</span>"#, escape_html(tag)));
        }
        row.push_str("</span>");
    }

    // سن
    row.push_str(&format!(r#"<span class="meta">{}</span>"#, escape_html(&age)));
    row.push_str("</li>\n");
    row
}

/// یک کارت آماری.
fn stat_card(label: &str, value: impl std::fmt::Display, color: &str) -> String {
    let style = if color.is_empty() { String::new() } else { format!(" style=\"color:{color}\"") };
    format!(
        r#"<div class="stat"><b{style}>{value}</b><small>{label}</small></div>"#,
        label = escape_html(label),
        value = value,
    )
}

/// نوار توزیع اولویت‌ها (مثل گزارش خط فرمان، ولی با HTML/CSS).
fn priority_bars(stats: &Stats) -> String {
    let mut out = String::new();
    let total = stats.total.max(1) as f64;
    for p in Priority::ALL {
        let n = stats.by_priority.get(&p).copied().unwrap_or(0);
        let percent = (n as f64 / total * 100.0).round();
        let class = priority_class(p);
        // نوارها با رنگ همان پیل اولویت همرنگ‌اند (کلاس مشترک)
        out.push_str(&format!(
            r#"<div class="row"><span class="k">{label}</span><span class="track"><i class="{cls}" style="width:{percent}%"></i></span><span class="n">{n}</span></div>"#,
            label = p.label(),
            cls = class,
            percent = percent,
            n = n,
        ));
    }
    out
}

/// ابر برچسب‌ها با تعداد (مرتب‌شده: پرتکرارترین اول).
fn tag_cloud(stats: &Stats) -> String {
    if stats.by_tag.is_empty() {
        return r#"<p class="muted">برچسبی ثبت نشده است.</p>"#.to_string();
    }
    // `by_tag` یک `HashMap` است → ترتیب ندارد؛ پس خودمان مرتب می‌کنیم
    // (همیشه: پرتکرارترین اول، و در تساوی الفبایی).
    let mut tags: Vec<(&String, &usize)> = stats.by_tag.iter().collect();
    tags.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    let mut out = String::from(r#"<div class="cloud">"#);
    for (tag, n) in tags {
        out.push_str(&format!(
            r#"<span class="tag">#{}<b>{}</b></span>"#,
            escape_html(tag),
            n
        ));
    }
    out.push_str("</div>");
    out
}

/// 🎨 استایل صفحه — یکجا و inline، تا فایل واقعاً «تک‌فایلی» باشد.
const STYLE: &str = r#"
  :root{
    --bg:#0e1116; --panel:#161b22; --panel2:#1c2230; --line:#2a3240;
    --fg:#e6edf3; --muted:#8b949e; --accent:#58a6ff; --green:#3fb950;
    --yellow:#d29922; --red:#f85149; --purple:#bc8cff; --radius:14px;
  }
  *{box-sizing:border-box}
  body{margin:0;background:linear-gradient(180deg,#0e1116,#0b0e13 60%);color:var(--fg);
       font-family:"Vazirmatn","Segoe UI",Tahoma,system-ui,sans-serif;min-height:100vh;padding:24px 16px 64px}
  .wrap{max-width:960px;margin:0 auto}
  header{display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap;margin-bottom:20px}
  h1{font-size:22px;margin:0;display:flex;align-items:center;gap:10px}
  h1 span.badge,h1 span.badge2{font-size:12px;font-weight:400;color:var(--muted);border:1px solid var(--line);padding:3px 8px;border-radius:999px}
  .dot{width:10px;height:10px;border-radius:50%;background:var(--green);box-shadow:0 0 0 4px rgba(63,185,80,.15)}
  .card{background:var(--panel);border:1px solid var(--line);border-radius:var(--radius);padding:16px;margin-bottom:16px}
  h2{font-size:14px;color:var(--muted);margin:0 0 12px;font-weight:600}
  .stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(130px,1fr));gap:12px}
  .stat{background:var(--panel2);border:1px solid var(--line);border-radius:10px;padding:12px}
  .stat b{display:block;font-size:24px;line-height:1.4}
  .stat small{color:var(--muted);font-size:12px}
  .bar{height:10px;background:var(--panel2);border-radius:999px;overflow:hidden;margin-top:12px;border:1px solid var(--line)}
  .bar > i{display:block;height:100%;background:linear-gradient(90deg,var(--accent),var(--green))}
  .two{display:grid;grid-template-columns:1.2fr 1fr;gap:16px}
  @media(max-width:700px){.two{grid-template-columns:1fr}}
  .row{display:flex;align-items:center;gap:10px;margin-bottom:8px;font-size:13px}
  .row .k{width:52px;color:var(--muted)}
  .track{flex:1;height:8px;background:var(--panel2);border-radius:999px;overflow:hidden;border:1px solid var(--line)}
  .track i{display:block;height:100%;background:var(--accent)}
  .track i.p-urgent{background:var(--red)} .track i.p-high{background:var(--yellow)}
  .track i.p-normal{background:var(--accent)} .track i.p-low{background:var(--muted)}
  .row .n{width:28px;text-align:left;font-variant-numeric:tabular-nums;color:var(--muted)}
  .cloud{display:flex;gap:6px;flex-wrap:wrap}
  .toolbar{display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-bottom:12px}
  select,input,button{font:inherit;border-radius:10px;border:1px solid var(--line);background:var(--panel2);color:var(--fg);padding:8px 10px;outline:none;font-size:13px}
  select:focus,input:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgba(88,166,255,.15)}
  ul{list-style:none;padding:0;margin:0}
  li.task{display:flex;align-items:center;gap:12px;padding:12px;border:1px solid var(--line);border-radius:12px;background:var(--panel2);margin-bottom:8px}
  li.task.done .title{text-decoration:line-through;color:var(--muted)}
  .title{flex:1;min-width:0;overflow-wrap:anywhere}
  .id{color:var(--muted);font-size:12px;font-variant-numeric:tabular-nums;white-space:nowrap}
  .pill{font-size:11px;padding:3px 8px;border-radius:999px;border:1px solid var(--line);white-space:nowrap}
  .p-urgent{color:var(--red);border-color:rgba(248,81,73,.4)}
  .p-high{color:var(--yellow);border-color:rgba(210,153,34,.4)}
  .p-normal{color:var(--accent);border-color:rgba(88,166,255,.4)}
  .p-low{color:var(--muted)}
  .tags{display:flex;gap:6px;flex-wrap:wrap}
  .tag{font-size:11px;color:var(--purple);background:rgba(188,140,255,.1);border:1px solid rgba(188,140,255,.25);padding:2px 7px;border-radius:999px}
  .tag b{margin-inline-start:5px;color:var(--fg)}
  .meta{color:var(--muted);font-size:11px;white-space:nowrap}
  .muted{color:var(--muted)}
  .hidden{display:none !important}
  footer{color:var(--muted);font-size:12px;text-align:center;margin-top:24px;line-height:2}
  code{background:var(--panel2);border:1px solid var(--line);border-radius:6px;padding:2px 6px;font-size:12px;direction:ltr;display:inline-block}
"#;

/// جاوااسکریپت کوچک: فقط فیلتر/جست‌وجو/شمارش.
///
/// 📌 عمداً هیچ `fetch`ی ندارد و هیچ محتوایی نمی‌سازد — چون همهٔ ردیف‌ها از
/// قبل در HTML هستند. تنها کارش اضافه/کم‌کردن کلاس `hidden` است.
const SCRIPT: &str = r#"
(function () {
  "use strict";
  var list = document.getElementById("list");
  var rows = Array.prototype.slice.call(list.querySelectorAll("li.task"));
  var count = document.getElementById("count");
  function apply() {
    var status = document.getElementById("f-status").value;
    var prio = document.getElementById("f-prio").value;
    var q = document.getElementById("f-q").value.trim().toLowerCase();
    var shown = 0;
    rows.forEach(function (li) {
      var ok = (status === "all" || li.dataset.status === status)
            && (prio === "all" || li.dataset.prio === prio)
            && (q === "" || li.dataset.text.indexOf(q) !== -1);
      li.classList.toggle("hidden", !ok);
      if (ok) shown++;
    });
    count.textContent = shown + " از " + rows.length + " تسک";
    document.getElementById("none").classList.toggle("hidden", shown !== 0);
  }
  ["f-status", "f-prio"].forEach(function (id) {
    document.getElementById(id).addEventListener("change", apply);
  });
  document.getElementById("f-q").addEventListener("input", apply);
  apply();
})();
"#;

/// ساخت کل صفحهٔ HTML از وضعیت فعلی برنامه.
pub fn render(app: &App) -> String {
    let stats = app.stats();
    let now = now_unix();

    // `String::with_capacity` جلوگیری از رشد پیاپی؛ حجم نهایی نامعلوم است،
    // پس یک برآورد محافظه‌کارانه می‌گیریم (هر تسک ≈ ۳۰۰ بایت HTML).
    let mut h = String::with_capacity(4_096 + app.tasks().len() * 400);

    h.push_str("<!DOCTYPE html>\n<html lang=\"fa\" dir=\"rtl\">\n<head>\n");
    h.push_str("<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    h.push_str("<title>ar — گزارش تسک‌ها</title>\n");
    h.push_str("<style>");
    h.push_str(STYLE);
    h.push_str("</style>\n</head>\n<body>\n<div class=\"wrap\">\n");

    // ── سرصفحه ──
    h.push_str("<header>\n<h1><span class=\"dot\"></span> ar ");
    h.push_str("<span class=\"badge\">گزارش ایستا — ساخته‌شده با Rust</span></h1>\n");
    let _ = write!(
        h,
        "<div class=\"muted\" style=\"font-size:12px;text-align:end\">انباره: {}<br>زمان ساخت: {}</div>\n",
        escape_html(&app.store_name()),
        format_datetime(now)
    );
    h.push_str("</header>\n");

    // ── کارت آمار ──
    h.push_str("<section class=\"card\">\n<div class=\"stats\">");
    h.push_str(&stat_card("کل تسک‌ها", stats.total, ""));
    h.push_str(&stat_card("انجام‌شده", stats.done, "var(--green)"));
    h.push_str(&stat_card("باز", stats.pending, "var(--yellow)"));
    h.push_str(&stat_card("فوریت کل", stats.urgency_total, "var(--red)"));
    h.push_str("</div>\n");
    let _ = write!(
        h,
        "<div class=\"bar\"><i style=\"width:{:.1}%\"></i></div>\n",
        stats.progress_percent
    );
    let avg = match stats.avg_seconds_to_done {
        // `Some(s)` = ثانیه‌ها؛ از همان تابع «انسان‌سازی» خودِ مدل استفاده می‌کنیم
        Some(s) => escape_html(&format!("میانگین زمان انجام: {}", humanize_age(s))),
        None => "میانگین زمان انجام: —".to_string(),
    };
    let _ = write!(
        h,
        "<p class=\"muted\" style=\"font-size:12px;margin:12px 0 0\">پیشرفت: {:.1}٪ · {}</p>\n",
        stats.progress_percent,
        avg
    );
    h.push_str("</section>\n");

    // ── دو ستون: توزیع اولویت + ابر برچسب ──
    h.push_str("<section class=\"card\">\n<div class=\"two\">\n<div>\n<h2>توزیع اولویت‌ها</h2>\n");
    h.push_str(&priority_bars(&stats));
    h.push_str("</div>\n<div>\n<h2>برچسب‌ها</h2>\n");
    h.push_str(&tag_cloud(&stats));
    h.push_str("</div>\n</div>\n</section>\n");

    // ── فهرست تسک‌ها ──
    h.push_str("<section class=\"card\">\n<h2>تسک‌ها</h2>\n");
    h.push_str(
        "<div class=\"toolbar\">\n\
         <select id=\"f-status\"><option value=\"all\">همه</option>\
         <option value=\"pending\">باز</option><option value=\"done\">انجام‌شده</option></select>\n\
         <select id=\"f-prio\"><option value=\"all\">همهٔ اولویت‌ها</option>\
         <option value=\"urgent\">فوری</option><option value=\"high\">بالا</option>\
         <option value=\"normal\">معمولی</option><option value=\"low\">پایین</option></select>\n\
         <input id=\"f-q\" placeholder=\"جست‌وجو در عنوان و برچسب…\">\n\
         <span class=\"pill\" id=\"count\"></span>\n\
         </div>\n",
    );

    h.push_str("<ul id=\"list\">\n");
    // مرتب‌سازی: بر اساس فوریت (فوری‌ترین و قدیمی‌ترین اول) — منطق خودِ برنامه
    let mut tasks: Vec<&Task> = app.tasks().iter().collect();
    tasks.sort_by(|a, b| a.urgency_cmp(b));
    for task in &tasks {
        h.push_str(&task_row(task, now));
    }
    h.push_str("</ul>\n");
    h.push_str("<p class=\"muted hidden\" id=\"none\" style=\"text-align:center;padding:20px\">چیزی با این فیلتر پیدا نشد.</p>\n");
    h.push_str("</section>\n");

    // ── پانویس ──
    h.push_str(
        "<footer>\n\
         این فایل با دستور <code>ar html</code> ساخته شده — با <b>Rust</b> و <b>صفر وابستگی</b>.<br>\n\
         داده‌ها داخل خود فایل حک شده‌اند (<span class=\"muted\">این‌جا برعکس <code>include_str!</code> عمل کردیم</span>)،\n\
         پس آفلاین و بدون سرور باز می‌شود.<br>\n\
         برای نسخهٔ تعاملی (افزودن/حذف/تغییر): <code>ar serve</code>\n\
         </footer>\n",
    );

    h.push_str("</div>\n<script>");
    h.push_str(SCRIPT);
    h.push_str("</script>\n</body>\n</html>\n");
    h
}

/// نوشتن گزارش روی دیسک؛ تعداد بایت‌های نوشته‌شده را برمی‌گرداند.
///
/// از همان الگوی امن `FileStore` استفاده می‌کنیم: نوشتن در فایل موقت و بعد
/// `rename` — تا اگر وسط کار برنامه بمیرد، یک فایل نیمه‌کاره باقی نماند.
pub fn write_report(app: &App, path: &Path) -> Result<usize> {
    let html = render(app);

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::io(format!("ساخت پوشهٔ «{}»", parent.display()), e))?;
    }

    let tmp = path.with_extension("html.tmp"); // «report.html.tmp» — نه overwrite خود فایل
    std::fs::write(&tmp, html.as_bytes())
        .map_err(|e| AppError::io(format!("نوشتن فایل موقت «{}»", tmp.display()), e))?;
    std::fs::rename(&tmp, path)
        .map_err(|e| AppError::io(format!("جابه‌جایی به «{}»", path.display()), e))?;

    Ok(html.len())
}

// ═══════════════════════════════════════════════════════════════════════════
//  تست‌ها
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(tasks: &[(&str, Priority)]) -> App {
        let mut app = App::open(crate::store::open_store(None)).unwrap();
        for (title, p) in tasks {
            app.add(title, *p, vec![]).unwrap();
        }
        app
    }

    #[test]
    fn escaping_covers_the_five_dangerous_chars() {
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("<b>"), "&lt;b&gt;");
        assert_eq!(escape_html("\"x\" 'y'"), "&quot;x&quot; &#39;y&#39;");
        // فارسی و ایموجی دست‌نخورده می‌مانند
        assert_eq!(escape_html("سلام 🌱"), "سلام 🌱");
        // ترتیب مهم: کاراکتر & ترجمه‌شده دوباره ترجمه نمی‌شود
        assert_eq!(escape_html("&lt;"), "&amp;lt;");
    }

    #[test]
    fn xss_attempt_is_neutralized() {
        let mut app = App::open(crate::store::open_store(None)).unwrap();
        // عنوانی که اگر خام درج شود، اسکریپت اجرا می‌کند
        app.add("<script>alert('XSS')</script>", Priority::Normal, vec![]).unwrap();

        let html = render(&app);
        assert!(!html.contains("<script>alert"), "عنوان باید فرار داده شود!");
        assert!(html.contains("&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;"));
        // تنها اسکریپتِ موجود، اسکریپت خودِ گزارش است
        assert_eq!(html.matches("<script>").count(), 1);
    }

    #[test]
    fn report_is_self_contained() {
        let app = app_with(&[("کار یک", Priority::High), ("کار دو", Priority::Low)]);
        let html = render(&app);

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("dir=\"rtl\""), "راست‌به‌چپ");
        // هیچ منبع بیرونی: بدون http، بدون fetch، بدون لینک CDN
        assert!(!html.contains("http://") && !html.contains("https://"), "باید آفلاین باشد");
        assert!(!html.contains("fetch("), "گزارش ایستا هیچ درخواستی نمی‌زند");
        assert!(html.contains("<style>") && html.contains("<script>"));
        // همهٔ تسک‌ها در HTML هستند (حتی بدون JS دیده می‌شوند)
        assert_eq!(html.matches("<li class=\"task").count(), 2);
        assert!(html.contains("کار یک") && html.contains("کار دو"));
    }

    #[test]
    fn row_class_has_no_stray_space() {
        // رگرسیون ظاهری: `class="task "` بد است — یا «task» یا «task done»
        let app = app_with(&[("باز", Priority::Low)]);
        let html = render(&app);
        assert!(html.contains(r#"<li class="task" data-status="#), "باید class=\"task\" باشد");
        assert!(!html.contains(r#"class="task ""#), "فاصلهٔ اضافه در کلاس");

        let mut app = app;
        let id = app.tasks()[0].id;
        app.set_done(id, true).unwrap();
        let done_html = render(&app);
        assert!(done_html.contains(r#"<li class="task done" data-status="done""#));
    }

    #[test]
    fn priority_classes_map_to_css() {
        assert_eq!(priority_class(Priority::Urgent), "p-urgent");
        assert_eq!(priority_class(Priority::High), "p-high");
        assert_eq!(priority_class(Priority::Normal), "p-normal");
        assert_eq!(priority_class(Priority::Low), "p-low");
        // کلاس‌ها در CSS هم تعریف شده‌اند (رگرسیونِ «کلاس بدون استایل»)
        for cls in ["p-urgent", "p-high", "p-normal", "p-low"] {
            assert!(STYLE.contains(cls), "کلاس {cls} در استایل نیست");
        }
    }

    #[test]
    fn dates_match_known_days() {
        // مبدأ یونیکس
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // روز کبیسه: ۲۰۰۰-۰۲-۲۹ (سال ۲۰۰۰ کبیسه است، برخلاف ۱۹۰۰)
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        // ۲۰۲۶-۱۰-۱۴ = روز ۲۰۷۴۰ (با پایتون هم چک شد: date(2026,10,14) — date(1970,1,1))
        assert_eq!(civil_from_days(20_740), (2026, 10, 14));
        // و روز ۲۰۷۵۳ همان ۲۰۲۶-۱۰-۲۷ است (مرز ماه)
        assert_eq!(civil_from_days(20_753), (2026, 10, 27));
        // تست کامل ساعت: ۲۰۲۳-۱۱-۱۴ ۲۲:۱۳:۲۰ UTC
        assert_eq!(datetime_utc(1_700_000_000), (2023, 11, 14, 22, 13, 20));
        assert_eq!(format_datetime(1_700_000_000), "2023-11-14 22:13");
    }

    #[test]
    fn write_report_is_atomic_and_reports_size() {
        let app = app_with(&[("فایل‌نویسی", Priority::Urgent)]);
        let dir = std::env::temp_dir().join(format!("ar-report-{}", std::process::id()));
        let path = dir.join("nested/report.html"); // پوشهٔ والد وجود ندارد → باید بسازد

        let bytes = write_report(&app, &path).expect("نوشتن");
        let text = std::fs::read_to_string(&path).expect("خواندن");
        assert_eq!(bytes, text.len());
        assert!(text.contains("فایل‌نویسی"));
        // فایل موقت نباید باقی بماند (rename آن را «مصرف» می‌کند)
        assert!(!path.with_extension("html.tmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
