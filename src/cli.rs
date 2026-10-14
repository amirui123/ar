//! # 🖥️ خط فرمان — پارس آرگومان‌ها بدون هیچ وابستگی
//!
//! در دنیای واقعی برای این کار از crateهایی مثل `clap` یا `argh` استفاده می‌کنند.
//! ولی اینجا خودمان می‌نویسیم؛ چون:
//! * یاد می‌گیری `env::args()`، `Iterator`، `HashMap` و `FromStr` چطور به هم می‌بندند.
//! * می‌بینی چرا امضای `T: FromStr` (کارِ عمومی/generic) نوشتن کد را خشک نمی‌کند.
//!
//! ## قواعدی که این پارسر پشتیبانی می‌کند
//! ```text
//! ar add "خرید نان" --priority high --tags food,خانه
//! ar list --status pending --limit 5 --json
//! ar done 3            # «3» یک آرگومان مکانی (positional) است
//! ar serve --port=8080 # شکل --کلید=مقدار هم پذیرفته می‌شود
//! ar list -- --این-یک-عنوان-است   # بعد از «--» همه‌چیز مکانی است
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use crate::errors::{AppError, Result};

/// سوییچ‌های شناخته‌شده: این‌ها **هیچ‌وقت مقدار نمی‌گیرند**.
///
/// 🤔 چرا لازم است؟ پارسر ما قانون «اگر توکن بعدی فلگ نبود، یعنی مقدار است»
/// را دارد. ولی در `ar --no-color seed`، توکن بعدی («seed») فلگ نیست و
/// اشتباهاً مقدار `--no-color` حساب می‌شد!
///
/// 📌 در دنیای واقعی این مشکل را با «تعریف طرح‌واره» حل می‌کنند (کاری که
/// `clap` می‌کند): برای هر زیرفرمان، فهرست فلگ‌ها و نوعشان را اعلام می‌کنی.
/// اینجا برای سادگی، یک فهرست سراسری از سوییچ‌های بی‌مقدار داریم —
/// و اگر سوییچ ناشناخته‌ای بیاید که مقدار داشته باشد، همان قانون قبلی عمل می‌کند.
const KNOWN_SWITCHES: &[&str] = &[
    "no-color", "no-colour", "color", "colour",
    "json", "all", "reverse", "r", "memory", "m",
    "force", "f", "help", "h", "quiet", "verbose",
];

/// آرگومان‌های پارس‌شده.
#[derive(Debug, Default, Clone)]
pub struct Args {
    /// نام برنامه (اولین آرگومان `argv`)
    pub program: String,
    /// زیرفرمان: اولین آرگومان مکانی (مثل `add`, `list`, `serve`)
    pub command: Option<String>,
    /// آرگومان‌های مکانی (بعد از حذف زیرفرمان)
    pub positional: Vec<String>,
    /// فلگ‌های دارای مقدار: `--priority high` → `{"priority": "high"}`
    pub flags: HashMap<String, String>,
    /// سوییچ‌های بی‌مقدار: `--json` → `{"json"}`
    pub switches: HashSet<String>,
}

impl Args {
    /// از `argv` واقعی سیستم پارس کن.
    pub fn parse_env() -> Result<Self> {
        // `std::env::args()` روی رشته‌های نامعتبر UTF-8 پنیک می‌کند؛
        // `args_os` امن است ولی اینجا سادگی را انتخاب می‌کنیم.
        Self::parse_from(std::env::args())
    }

    /// پارس از هر پیمایش‌گر رشته (برای تست راحت است).
    ///
    /// # خطاها
    /// اگر فلگ ناشناخته باشد یا مقدارش جا افتاده باشد، [`AppError::Usage`] می‌دهد.
    pub fn parse_from<I>(iter: I) -> Result<Self>
    where
        I: IntoIterator<Item = String>,
    {
        // `.peekable()` = پیمایش‌گری که می‌تواند «یک قدم جلوتر» را ببیند بدون مصرف.
        // کلید همین فایل است: برای فهمیدن اینکه `--port` مقدار دارد یا نه،
        // باید ببینیم توکن بعدی چیست.
        let mut it = iter.into_iter().peekable();
        let mut args = Args::default();

        args.program = it.next().unwrap_or_else(|| "ar".to_string());

        // ── حالت‌های داخلی پارسر ──
        let mut after_double_dash = false; // بعد از «--» همه‌چیز مکانی است

        while let Some(token) = it.next() {
            // ۱) جداکنندهٔ پایان فلگ‌ها
            if !after_double_dash && token == "--" {
                after_double_dash = true;
                continue;
            }

            // ۲) فلگ/سوییچ؟ (با - شروع می‌شود و فقط یک «-» نیست)
            if !after_double_dash && token.starts_with('-') && token != "-" {
                // حالت --کلید=مقدار
                if let Some((key, value)) = token.split_once('=') {
                    args.flags.insert(normalize_key(key), value.to_string());
                    continue;
                }

                let key = normalize_key(&token);

                // آیا این کلید یک سوییچ بی‌مقدار شناخته‌شده است؟
                if KNOWN_SWITCHES.contains(&key.as_str()) {
                    args.switches.insert(key);
                    continue;
                }

                // آیا توکن بعدی «مقدار» است یا فلگ بعدی؟
                // قاعده: اگر توکن بعدی وجود دارد و *فلگ نیست*، مقدار است.
                let next_is_value = match it.peek() {
                    None => false,
                    Some(next) => !is_flag_like(next),
                };

                if next_is_value {
                    // `it.next()` قطعاً Some است چون peek گفت
                    let value = it.next().unwrap();
                    args.flags.insert(key, value);
                } else {
                    // سوییچ بی‌مقدار مثل `--json`
                    args.switches.insert(key);
                }
                continue;
            }

            // ۳) آرگومان مکانی
            if args.command.is_none() {
                args.command = Some(token);
            } else {
                args.positional.push(token);
            }
        }

        Ok(args)
    }

    // ─────────────────────────────────────────────────────────────────────
    //  دسترسی‌ها (accessors)
    // ─────────────────────────────────────────────────────────────────────

    /// زیرفرمان (اگر داده شده باشد).
    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }

    /// آرگومان مکانی شمارهٔ `i`.
    pub fn arg(&self, i: usize) -> Option<&str> {
        self.positional.get(i).map(String::as_str)
    }

    /// مقدار فلگ — نام‌های جایگزین (alias) را هم می‌پذیرد.
    ///
    /// مثال: `args.flag(&["priority", "p"])` هم `--priority` و هم `-p` را می‌گیرد.
    pub fn flag(&self, names: &[&str]) -> Option<&str> {
        names.iter().find_map(|n| self.flags.get(*n).map(String::as_str))
    }

    /// مثل [`Args::flag`] ولی با مقدار پیش‌فرض.
    ///
    /// ⚠️ نکتهٔ عمرها (lifetimes): ورودی `default` و خروجی هم‌عمر `'a` هستند،
    /// پس برمی‌گرداندن مستقیم `default` بدون هیچ کپی‌ای ممکن است. این همان چیزی است
    /// که کامپایلر Rust را برای کتابخانه‌نویسی «بی‌هزینه» می‌کند.
    pub fn flag_or<'a>(&'a self, names: &[&str], default: &'a str) -> &'a str {
        self.flag(names).unwrap_or(default)
    }

    /// آیا سوییچ یا فلگ با این نام وجود دارد؟
    pub fn has(&self, names: &[&str]) -> bool {
        names.iter().any(|n| self.switches.contains(*n) || self.flags.contains_key(*n))
    }

    /// آیا سوییچ بی‌مقدار داده شده؟
    pub fn switch(&self, names: &[&str]) -> bool {
        names.iter().any(|n| self.switches.contains(*n))
    }

    /// خواندن یک فلگ به‌صورت عدد (با هر نوعی که `FromStr` دارد).
    ///
    /// 🌟 **این امضا را حفظ کن — الگوی طلایی Rust است:**
    /// `T: FromStr, T::Err: fmt::Display` یعنی هم `u16`، هم `f64`، هم `usize`
    /// و هم نوع خودت کار می‌کند، بدون اینکه هیچ کد تکراری بنویسی.
    pub fn number<T>(&self, names: &[&str]) -> Result<Option<T>>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        match self.flag(names) {
            None => Ok(None),
            Some(raw) => raw.trim().parse::<T>().map(Some).map_err(|e| {
                AppError::BadInput(format!(
                    "مقدار «{raw}» برای «--{}» عدد معتبر نیست: {e}",
                    names.first().copied().unwrap_or("?")
                ))
            }),
        }
    }

    /// تقسیم یک فلگ ویرگولی به چند مقدار (مثل `--tags a,b,c`).
    /// هر عضو خالی حذف می‌شود و فاصله‌ها trim می‌شوند.
    pub fn list(&self, names: &[&str]) -> Vec<String> {
        self.flag(names)
            .map(|raw| {
                raw.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// `--priority` و `-p` و `p` را یکسان می‌کند (کلید بدون خط‌تیره).
fn normalize_key(raw: &str) -> String {
    raw.trim_start_matches('-').to_string()
}

/// آیا این توکن «شبیه فلگ» است؟ (یعنی به‌عنوان مقدار مصرف نمی‌شود)
///
/// ترفند ریز: `-1` و `-3.5` عدد منفی هستند و باید مقدار حساب شوند،
/// ولی `-p` فلگ است. تشخیص با تلاش برای پارس به‌عنوان `f64`.
fn is_flag_like(token: &str) -> bool {
    if !token.starts_with('-') {
        return false;
    }
    // اگر بعد از خط‌تیره عددی باشد (مثل -5 یا -2.5) → مقدار است، نه فلگ.
    !token[1..].parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &[&str]) -> Result<Args> {
        Args::parse_from(s.iter().map(|x| (*x).to_string()))
    }

    #[test]
    fn command_and_positional() {
        let a = parse(&["ar", "add", "خرید نان"]).unwrap();
        assert_eq!(a.command(), Some("add"));
        assert_eq!(a.arg(0), Some("خرید نان"));
    }

    #[test]
    fn flags_in_both_forms() {
        let a = parse(&["ar", "list", "--limit", "5", "--status=done"]).unwrap();
        assert_eq!(a.flag(&["limit"]), Some("5"));
        assert_eq!(a.flag(&["status"]), Some("done"));
    }

    #[test]
    fn short_alias_normalizes_same() {
        let a = parse(&["ar", "add", "x", "-p", "high"]).unwrap();
        // هم -p و هم --p به کلید «p» نگاشت می‌شوند
        assert_eq!(a.flag(&["priority", "p"]), Some("high"));
    }

    #[test]
    fn global_switch_before_command_does_not_eat_it() {
        // این باگ واقعی بود: `--no-color` مقدار «seed» را می‌خورد!
        let a = parse(&["ar", "--no-color", "seed"]).unwrap();
        assert!(a.switch(&["no-color"]));
        assert_eq!(a.command(), Some("seed"));

        let b = parse(&["ar", "--json", "list", "--limit", "5"]).unwrap();
        assert!(b.switch(&["json"]));
        assert_eq!(b.command(), Some("list"));
        assert_eq!(b.flag(&["limit"]), Some("5"));
    }

    #[test]
    fn switches_without_value() {
        let a = parse(&["ar", "list", "--json", "--all"]).unwrap();
        assert!(a.switch(&["json"]));
        assert!(a.switch(&["all"]));
        assert!(!a.switch(&["nope"]));
    }

    #[test]
    fn negative_numbers_are_values_not_flags() {
        let a = parse(&["ar", "list", "--limit", "-1"]).unwrap();
        assert_eq!(a.flag(&["limit"]), Some("-1"));
    }

    #[test]
    fn double_dash_stops_flag_parsing() {
        let a = parse(&["ar", "add", "--", "--این-عنوان-است"]).unwrap();
        assert_eq!(a.arg(0), Some("--این-عنوان-است"));
        assert!(a.switches.is_empty());
    }

    #[test]
    fn generic_number_parsing() {
        let a = parse(&["ar", "list", "--limit", "7", "--ratio", "0.5"]).unwrap();
        assert_eq!(a.number::<usize>(&["limit"]).unwrap(), Some(7));
        assert_eq!(a.number::<f64>(&["ratio"]).unwrap(), Some(0.5));
        assert_eq!(a.number::<u16>(&["missing"]).unwrap(), None);
        assert!(a.number::<usize>(&["ratio"]).is_err()); // «0.5» عدد صحیح نیست
    }

    #[test]
    fn list_splitting() {
        let a = parse(&["ar", "add", "x", "--tags", " a , b ,, c "]).unwrap();
        assert_eq!(a.list(&["tags"]), vec!["a", "b", "c"]);
    }
}
