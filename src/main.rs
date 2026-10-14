//! # 🚀 نقطهٔ شروع برنامه (`ar`)
//!
//! این فایل، «لایهٔ نمایش» (presentation) است: آرگومان‌ها را می‌گیرد،
//! منطق را از [`ar::app`] صدا می‌زند، و نتیجه را چاپ می‌کند.
//! **هیچ منطق کاری اینجا نوشته نمی‌شود** — این تفکیک، تست‌پذیری می‌آورد.
//!
//! ## الگوی `main` در Rust
//! ```text
//! fn main() -> ExitCode     ← کد خروج را می‌تواند برگرداند
//!     └── run() -> Result<ExitCode>   ← تمام منطق با ?
//! ```
//! چرا؟ چون `main` نمی‌تواند `Result` با نوع خطای دلخواه بگیرد و
//! `?` بزند *و* همزمان پیام دوستانه بدهد. پس منطق را به `run` می‌بریم
//! و در `main` فقط خطا را «زیبا چاپ» می‌کنیم.

use std::path::PathBuf;
use std::process::ExitCode;

use ar::app::{Filter, SortKey, StatusFilter};
use ar::cli::Args;
use ar::color;
use ar::model::{Priority, TaskId, humanize_age, now_unix};
use ar::store::default_data_path;
use ar::table::{self, Table};
use ar::{App, AppError, Result};

fn main() -> ExitCode {
    // همهٔ جادو در run() است؛ اینجا فقط تصمیم‌گیری دربارهٔ «چه چاپ کنم».
    match run() {
        Ok(code) => code,
        Err(err) => {
            // ⚠️ خطاها به stderr می‌روند (نه stdout) تا بشود خروجی داده را هدایت کرد.
            eprintln!("{} {err}", color::red("✖"));
            // «زنجیرهٔ علت» را نشان بده — ترفندی که خطاهای تودرتو را شفاف می‌کند
            let mut source = std::error::Error::source(&err);
            while let Some(cause) = source {
                eprintln!("  {} {cause}", color::dim("↳"));
                source = cause.source();
            }
            ExitCode::from(err.exit_code() as u8)
        }
    }
}

/// منطق اصلی برنامه.
fn run() -> Result<ExitCode> {
    let args = Args::parse_env()?;

    // ── تنظیمات سراسری ────────────────────────────────────────────────────
    // ترتیب مهم است: اول رنگ را تنظیم کن تا همهٔ چاپ‌های بعدی درست باشند.
    if args.switch(&["no-color", "no-colour"]) {
        color::set_enabled(false);
    } else if args.switch(&["color"]) {
        color::set_enabled(true);
    }

    // مسیر داده: `--data` > متغیر محیطی AR_DATA > data/tasks.json
    let data_path: Option<PathBuf> = args.flag(&["data", "d"]).map(PathBuf::from);

    match args.command() {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(ExitCode::SUCCESS)
        }
        Some("version") => {
            println!("{}", ar::version_line());
            Ok(ExitCode::SUCCESS)
        }
        // ── همهٔ زیرفرمان‌ها که به داده نیاز دارند ──
        Some(cmd) => {
            // برای دستورهای نمایشی/آزمایشی نیازی به باز کردن انباره نیست
            let store_path = match data_path {
                Some(p) => Some(p),
                None => {
                    if args.switch(&["memory", "m"]) {
                        None
                    } else {
                        Some(default_data_path())
                    }
                }
            };

            let mut app = App::open(ar::store::open_store(store_path))?;

            match cmd {
                "add" => cmd_add(&mut app, &args),
                "list" | "ls" => cmd_list(&app, &args),
                "done" => cmd_set_done(&mut app, &args, true),
                "undo" => cmd_set_done(&mut app, &args, false),
                "rm" | "remove" | "del" => cmd_remove(&mut app, &args),
                "purge" => cmd_purge(&mut app, &args),
                "stats" => cmd_stats(&app, &args),
                "export" => cmd_export(&app, &args),
                "html" | "report" => cmd_html(&app, &args),
                "seed" => cmd_seed(&mut app, &args),
                "serve" => ar::demos::serve(app, &args),
                "demo" => ar::demos::run_demo(&args),
                other => Err(AppError::Usage(format!(
                    "زیرفرمان «{other}» را نمی‌شناسم.\n\nراهنما را با `ar help` ببین."
                ))),
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  زیرفرمان‌ها
// ═══════════════════════════════════════════════════════════════════════════

/// `ar add <عنوان> [--priority p] [--tags a,b]`
fn cmd_add(app: &mut App, args: &Args) -> Result<ExitCode> {
    // عنوان می‌تواند چند کلمه باشد اگر کاربر نقل‌قول گذاشته باشد؛
    // اگر نه، همهٔ آرگومان‌های مکانی را با فاصله به هم می‌چسبانیم (راحت‌تر برای کاربر).
    let title = if args.positional.is_empty() {
        return Err(AppError::Usage(
            "استفاده: ar add «عنوان تسک» [--priority high] [--tags کار,خانه]".into(),
        ));
    } else {
        args.positional.join(" ")
    };

    let priority: Priority = match args.flag(&["priority", "p"]) {
        Some(raw) => raw.parse()?, // ← `FromStr` که در model.rs نوشتیم
        None => Priority::default(),
    };

    let tags = args.list(&["tags", "t"]);
    let id = app.add(&title, priority, tags)?;
    let wrote = app.save()?;

    println!("{} تسک {} ساخته شد: «{}»", color::green("✔"), id, title);
    if !wrote {
        println!("{}", color::dim("(بدون تغییر برای ذخیره)"));
    }
    Ok(ExitCode::SUCCESS)
}

/// `ar list [--status ...] [--tag ...] [--search ...] [--sort ...] [--limit n] [--json]`
fn cmd_list(app: &App, args: &Args) -> Result<ExitCode> {
    let filter = Filter {
        status: match args.flag(&["status", "s"]) {
            Some(raw) => StatusFilter::parse(raw)?,
            None => StatusFilter::All,
        },
        tag: args.flag(&["tag", "t"]).map(str::to_string),
        search: args.flag(&["search", "q"]).map(str::to_string),
        sort: match args.flag(&["sort"]) {
            Some(raw) => SortKey::parse(raw)?,
            None => SortKey::Id,
        },
        reverse: args.switch(&["reverse", "r"]),
        limit: args.number::<usize>(&["limit", "n"])?,
    };

    let rows = app.list(&filter);

    // ── خروجی JSON (برای اتصال به ابزارهای دیگر) ──
    if args.switch(&["json"]) {
        let value = ar::json::Json::Arr(rows.iter().map(|t| ar::store::task_to_json(t)).collect());
        println!("{}", value.to_pretty(2));
        return Ok(ExitCode::SUCCESS);
    }

    if rows.is_empty() {
        println!("{}", color::dim("هیچ تسکی با این شرط‌ها پیدا نشد."));
        return Ok(ExitCode::SUCCESS);
    }

    // ── جدول زیبا ──
    let now = now_unix();
    let mut t = Table::new(["#", "عنوان", "اولویت", "برچسب‌ها", "سن", "وضعیت"]).right(0);
    for task in &rows {
        let (status, icon) = if task.done {
            (color::green("انجام‌شده"), "✔")
        } else {
            (color::yellow("باز"), "☐")
        };

        // عنوان را کوتاه می‌کنیم تا جدول از عرض ترمینال بیرون نزند
        let title = table::truncate(&task.title, 42);
        // عنوان تسکِ انجام‌شده را «کم‌رنگ» نشان بده (ترفند UX)
        let title = if task.done { color::dim(&title) } else { title };

        let tags = if task.tags.is_empty() {
            color::dim("—")
        } else {
            color::cyan(&task.tags.join(","))
        };

        // اولویت با رنگ خودش
        let prio = color::paint(task.priority.label(), task.priority.color_code());

        t.push([
            task.id.get().to_string(),
            format!("{icon} {title}"),
            prio,
            tags,
            color::dim(&humanize_age(now.saturating_sub(task.created_at))),
            status,
        ]);
    }

    print!("{}", t.render());
    println!("{}", color::dim(&format!("({} تسک، انباره: {})", rows.len(), app.store_name())));
    Ok(ExitCode::SUCCESS)
}

/// `ar done <#>...` و `ar undo <#>...`
fn cmd_set_done(app: &mut App, args: &Args, done: bool) -> Result<ExitCode> {
    let ids = parse_ids(app, args)?;
    if ids.is_empty() {
        return Err(AppError::Usage(format!(
            "استفاده: ar {} <شناسه> [شناسه دوم …]",
            if done { "done" } else { "undo" }
        )));
    }

    // `for` روی `Vec<TaskId>` — چون `TaskId: Copy` است، ایندکس‌ها همیشه معتبر می‌مانند
    for id in ids {
        let changed = app.set_done(id, done)?;
        let task = app.get(id)?;
        let verb = if done { "انجام شد" } else { "به حالت باز برگشت" };
        if changed {
            println!("{} {} {} — {}", color::green("✔"), id, verb, task.title);
        } else {
            println!("{} {} از قبل در همین وضعیت بود", color::dim("•"), id);
        }
    }

    app.save()?;
    Ok(ExitCode::SUCCESS)
}

/// `ar rm <#>...`
fn cmd_remove(app: &mut App, args: &Args) -> Result<ExitCode> {
    let ids = parse_ids(app, args)?;
    if ids.is_empty() {
        return Err(AppError::Usage("استفاده: ar rm <شناسه>".into()));
    }

    // الگوی مهم: عملیات «مخرب» را بدون تأیید جدا انجام نده، مگر با --force.
    // (اینتراکتیو نگه‌داشتن ساده است: اگر --force نبود، فقط نشان بده که چه می‌شود)
    let force = args.switch(&["force", "f"]);

    if !force {
        println!("{}", color::yellow("این تسک‌ها حذف خواهند شد (برای انجام: --force):"));
        for id in &ids {
            if let Ok(t) = app.get(*id) {
                println!("  {id} {}", table::truncate(&t.title, 60));
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    for id in ids {
        let task = app.remove(id)?;
        println!("{} {} حذف شد: {}", color::red("✖"), id, table::truncate(&task.title, 50));
    }
    app.save()?;
    Ok(ExitCode::SUCCESS)
}

/// `ar purge` — پاک‌سازی همهٔ انجام‌شده‌ها
fn cmd_purge(app: &mut App, _args: &Args) -> Result<ExitCode> {
    let n = app.purge_done();
    if n == 0 {
        println!("{}", color::dim("چیزی برای پاک‌سازی نبود."));
    } else {
        app.save()?;
        println!("{} {n} تسک انجام‌شده پاک شد.", color::green("✔"));
    }
    Ok(ExitCode::SUCCESS)
}

/// `ar stats [--json]`
fn cmd_stats(app: &App, args: &Args) -> Result<ExitCode> {
    let s = app.stats();

    if args.switch(&["json"]) {
        let value = ar::json::Json::obj([
            ("total", ar::json::Json::from(s.total)),
            ("done", ar::json::Json::from(s.done)),
            ("pending", ar::json::Json::from(s.pending)),
            ("progress_percent", ar::json::Json::from(s.progress_percent)),
            (
                "by_priority",
                ar::json::Json::Obj(
                    s.by_priority
                        .iter()
                        .map(|(p, n)| (p.as_str().to_string(), ar::json::Json::from(*n)))
                        .collect(),
                ),
            ),
            (
                "by_tag",
                ar::json::Json::Obj(
                    // HashMap ترتیب ندارد → برای خروجی پایدار، کلیدها را مرتب می‌کنیم
                    {
                        let mut pairs: Vec<_> = s.by_tag.iter().collect();
                        pairs.sort_by(|a, b| a.0.cmp(b.0));
                        pairs
                            .into_iter()
                            .map(|(k, v)| (k.clone(), ar::json::Json::from(*v)))
                            .collect()
                    },
                ),
            ),
        ]);
        println!("{}", value.to_pretty(2));
        return Ok(ExitCode::SUCCESS);
    }

    println!("{}", table::section("آمار تسک‌ها"));
    println!("  کل: {}   {}   {}", 
        color::bold(&s.total.to_string()),
        color::green(&format!("انجام‌شده: {}", s.done)),
        color::yellow(&format!("باز: {}", s.pending)),
    );
    println!("  پیشرفت: {}", table::progress_bar(s.progress_percent, 30));
    println!("  فوریت کل (مجموع وزن اولویت‌ها): {}", color::magenta(&s.urgency_total.to_string()));

    if let Some(avg) = s.avg_seconds_to_done {
        println!("  میانگین زمان تا انجام‌شدن: {}", color::cyan(&humanize_age(avg)));
    }
    if let Some(id) = s.oldest_pending {
        println!("  قدیمی‌ترین تسک باز: {}", color::yellow(&id.to_string()));
    }
    println!();

    // جدول اولویت‌ها — BTreeMap همیشه مرتب پیمایش می‌شود
    println!("{}", table::section("به تفکیک اولویت"));
    let mut t = Table::new(["اولویت", "تعداد", "سهم"]).right(1);
    for (prio, count) in &s.by_priority {
        let share = if s.total == 0 { 0.0 } else { (*count as f64) * 100.0 / (s.total as f64) };
        t.push([
            color::paint(prio.label(), prio.color_code()),
            count.to_string(),
            table::progress_bar(share, 12),
        ]);
    }
    print!("{}", t.render());

    if !s.by_tag.is_empty() {
        println!("{}", table::section("به تفکیک برچسب"));
        let mut tags: Vec<(&String, &usize)> = s.by_tag.iter().collect();
        // مرتب‌سازی نزولی بر اساس تعداد، سپس الفبایی — نمونهٔ کامل `sort_by`
        tags.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        let mut t = Table::new(["برچسب", "تعداد"]).right(1);
        for (tag, count) in tags {
            t.push([color::cyan(tag), count.to_string()]);
        }
        print!("{}", t.render());
    }

    Ok(ExitCode::SUCCESS)
}

/// `ar export` — چاپ کل داده‌ها به‌صورت JSON (برای پشتیبان‌گیری)
fn cmd_export(app: &App, _args: &Args) -> Result<ExitCode> {
    let value = ar::json::Json::Arr(app.tasks().iter().map(ar::store::task_to_json).collect());
    println!("{}", value.to_pretty(2));
    Ok(ExitCode::SUCCESS)
}

/// `ar seed` — پر کردن با دادهٔ نمونه
/// `ar html [--out report.html]` — ساخت گزارش HTML ایستا (بدون سرور).
///
/// 🎯 این زیرفرمان نشان می‌دهد «HTML» فقط کار سرور وب نیست: خودِ برنامه
/// می‌تواند یک فایل تک‌تکه بسازد که همه‌جا و آفلاین باز می‌شود.
fn cmd_html(app: &App, args: &Args) -> Result<ExitCode> {
    let out = PathBuf::from(args.flag_or(&["out", "o"], "report.html"));

    let bytes = ar::report::write_report(app, &out)?;

    println!(
        "{} گزارش HTML ساخته شد: {}",
        color::green("✔"),
        color::bold(&out.display().to_string())
    );
    println!("  حجم: {} بایت · {} تسک", bytes, app.tasks().len());
    println!("  {}", color::dim("این فایل مستقل است — بدون سرور و آفلاین باز می‌شود."));
    Ok(ExitCode::SUCCESS)
}

fn cmd_seed(app: &mut App, _args: &Args) -> Result<ExitCode> {
    let n = app.seed()?;
    app.save()?;
    println!("{} {n} تسک نمونه اضافه شد.", color::green("✔"));
    Ok(ExitCode::SUCCESS)
}

// ═══════════════════════════════════════════════════════════════════════════
//  کمک‌کننده‌ها
// ═══════════════════════════════════════════════════════════════════════════

/// شناسه‌ها را از آرگومان‌های مکانی بخوان.
///
/// اگر هیچ شناسه‌ای ندهی، همهٔ تسک‌ها (برای `purge` و شبیه آن) یا لیست خالی.
/// ترفند «راهنمای خطا»: اگر شناسه پیدا نشود، نزدیک‌ترین‌ها را پیشنهاد می‌دهیم.
fn parse_ids(app: &App, args: &Args) -> Result<Vec<TaskId>> {
    let mut ids = Vec::with_capacity(args.positional.len());
    for raw in &args.positional {
        match raw.parse::<TaskId>() {
            Ok(id) => ids.push(id),
            Err(_) => {
                return Err(AppError::BadInput(format!("شناسهٔ «{raw}» عدد نیست")));
            }
        }
    }

    // اعتبارسنجی وجود شناسه‌ها + پیشنهاد
    for id in &ids {
        if let Err(AppError::TaskNotFound(_)) = app.get(*id) {
            let suggestions = app.suggest_ids(*id, 3);
            let hint = suggestions
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("، ");
            return Err(AppError::BadInput(format!(
                "تسکی با شناسهٔ {id} وجود ندارد.{}",
                if hint.is_empty() {
                    String::new()
                } else {
                    format!(" شاید منظورت یکی از این‌ها بود: {hint}")
                }
            )));
        }
    }

    Ok(ids)
}

/// چاپ راهنما.
///
/// 📌 نکتهٔ آموزشی: نسخهٔ اول این تابع از `{h("ar")}` داخل رشتهٔ فرمت استفاده
/// می‌کرد که **خطای کامپایل** می‌دهد — در `format!` فقط می‌توانی نام متغیر یا
/// اندیس یا نام‌آرگومان بگذاری، نه عبارت دلخواه. اینجا با `{}` و آرگومان واقعی می‌نویسیم.
fn print_help() {
    let b = color::bold;
    let d = color::dim;

    println!("{} — مدیر تسک خط فرمان + آزمایشگاه Rust  {}", b("ar"), d(&format!("(نسخهٔ {})", ar::VERSION)));
    println!();
    println!("{}", b("استفاده:"));
    println!("  {} ar <زیرفرمان> [آرگومان‌ها] [فلگ‌ها]", d("$"));
    println!();
    println!("{}", b("زیرفرمان‌ها:"));
    println!("  {}          افزودن تسک      {}", b("add <عنوان>"), d("[--priority p] [--tags a,b]"));
    println!("  {}             نمایش تسک‌ها  {}", b("list, ls"), d("[--status s] [--tag t] [--search q] [--sort k] [--limit n] [--reverse] [--json]"));
    println!("  {}            انجام‌شده کردن", b("done <#>…"));
    println!("  {}            برگرداندن به حالت باز", b("undo <#>…"));
    println!("  {}              حذف  {}", b("rm <#>…"), d("(برای اجرای واقعی: --force)"));
    println!("  {}                پاک‌سازی همهٔ انجام‌شده‌ها", b("purge"));
    println!("  {}                آمار  {}", b("stats"), d("[--json]"));
    println!("  {}               چاپ همهٔ داده‌ها به‌صورت JSON", b("export"));
    println!("  {}           ساخت گزارش HTML مستقل  {}", b("html"), d("[--out report.html]"));
    println!("  {}                 افزودن ۵ تسک نمونه", b("seed"));
    println!("  {}                اجرای سرور وب  {}", b("serve"), d("[--port 8080] [--host 0.0.0.0]"));
    println!("  {}          دموی مفاهیم زبان  {}", b("demo <موضوع>"), d("(برای فهرست: ar demo)"));
    println!();
    println!("{}", b("فلگ‌های عمومی:"));
    println!("  {}        مسیر فایل داده  {}", b("--data <مسیر>"), d("(پیش‌فرض: data/tasks.json یا متغیر $AR_DATA)"));
    println!("  {}         استفاده از حافظهٔ موقت (هیچ‌چیز ذخیره نمی‌شود)", b("--memory, -m"));
    println!("  {}   کنترل رنگ خروجی  {}", b("--color / --no-color"), d("(به‌صورت خودکار با NO_COLOR هم سازگار است)"));
    println!("  {}           همین راهنما", b("-h, --help"));
    println!("  {}                    نسخه", b("version"));
    println!();
    println!("{}", b("نمونه‌ها:"));
    println!("  ar add \"نوشتن مستندات\" --priority high --tags کار,docs");
    println!("  ar list --status pending --sort urgency --limit 5");
    println!("  ar stats --json");
    println!("  ar serve --port 8080     {}", d("→ سپس در مرورگر باز کن"));
    println!("  ar demo ownership        {}", d("→ مالکیت، خط‌به‌خط"));
}

#[cfg(test)]
mod tests {
    //! تست‌های سطح باینری: فقط چیزهایی که به خروجی چاپ‌شده مربوط‌اند.
    //! (منطق اصلی در `src/app.rs` و `src/json.rs` تست شده است.)
    use super::*;

    #[test]
    fn help_does_not_panic() {
        print_help();
    }

    #[test]
    fn parse_ids_rejects_non_numbers() {
        let app = App::open(ar::store::open_store(None)).unwrap();
        let args = Args::parse_from(["ar", "done", "abc"].iter().map(|s| s.to_string())).unwrap();
        assert!(parse_ids(&app, &args).is_err());
    }
}
