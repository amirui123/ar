//! # 📊 جدول و چاپ زیبا
//!
//! ترمینال یک «بوم نقاشی متنی» است. برای ساختن جدول مرتب باید بدانیم هر سلول
//! چند کاراکتر *دیده‌شدنی* دارد. دو دام مهم:
//!
//! 1. **کدهای رنگ ANSI** کاراکتر واقعی نیستند ولی در `len()` شمرده می‌شوند.
//! 2. **بایت ≠ کاراکتر:** «سلام» در UTF-8 هشت بایت است ولی چهار کاراکتر.
//!
//! جواب درست: اول کدهای رنگ را حذف کن، بعد `chars().count()` بگیر
//! (و برای متن‌های خیلی جدی‌تر، عرض «گرافم» را حساب کن — به `docs/13` نگاه کن).

use std::fmt::Write as _;

use crate::color;

/// یک جدول ساده با کادر و ترازبندی.
#[derive(Debug, Clone, Default)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    /// ستون‌هایی که باید راست‌چین باشند (مثل اعداد)
    right_align: Vec<bool>,
}

impl Table {
    /// ساخت جدول با سرستون‌ها.
    ///
    /// 🌟 **امضای همه‌فن‌حریف:** `H: Into<String>` + `I: IntoIterator<Item = H>`
    /// یعنی هر چیزی که قابل پیمایش باشد و هر عنصری که به `String` تبدیل شود:
    /// `Table::new(["a", "b"])` یا `Table::new(vec![String::from("a")])` یا
    /// `Table::new(items.iter().map(|x| x.name.clone()))` — همه کار می‌کنند.
    pub fn new<H, I>(headers: I) -> Self
    where
        H: Into<String>,
        I: IntoIterator<Item = H>,
    {
        let headers: Vec<String> = headers.into_iter().map(Into::into).collect();
        let right_align = vec![false; headers.len()];
        Table { headers, rows: Vec::new(), right_align }
    }

    /// ستون `column` را راست‌چین کن (برای اعداد مناسب است).
    ///
    /// الگوی «سازندهٔ زنجیره‌ای» (builder): `self` را مصرف و برگردان تا
    /// `Table::new([...]).right(0).right(3)` ممکن باشد.
    #[must_use]
    pub fn right(mut self, column: usize) -> Self {
        if column < self.right_align.len() {
            self.right_align[column] = true;
        }
        self
    }

    /// تعداد سرستون‌ها.
    pub fn columns(&self) -> usize {
        self.headers.len()
    }

    /// افزودن ردیف. اگر تعداد سلول‌ها بیشتر از سرستون‌ها باشد، کوتاه می‌شود؛
    /// اگر کمتر باشد، با رشتهٔ خالی پر می‌شود (تا جدول هیچ‌وقت کج نشود).
    pub fn push<R, T>(&mut self, row: R)
    where
        T: Into<String>,
        R: IntoIterator<Item = T>,
    {
        let mut cells: Vec<String> = row.into_iter().map(Into::into).collect();
        cells.truncate(self.headers.len());
        while cells.len() < self.headers.len() {
            cells.push(String::new());
        }
        self.rows.push(cells);
    }

    /// آیا جدولی برای نمایش وجود دارد؟
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// عرض دیده‌شدنی یک رشته (بدون کدهای رنگ، بر اساس کاراکتر).
    fn visible_width(s: &str) -> usize {
        color::strip_ansi(s).chars().count()
    }

    /// یک سلول را به عرض `width` برسان (چپ‌چین یا راست‌چین).
    fn pad_cell(s: &str, width: usize, right: bool) -> String {
        let content = Self::visible_width(s);
        // `saturating_sub` تا اگر محتوای عجیبی بزرگ‌تر بود، پنیک نکنیم
        let fill = width.saturating_sub(content);
        let spaces = " ".repeat(fill);
        if right {
            format!("{spaces}{s}")
        } else {
            format!("{s}{spaces}")
        }
    }

    /// عرض هر ستون = بیشترین عرض دیده‌شدنی بین سرستون و سلول‌هایش.
    fn column_widths(&self) -> Vec<usize> {
        let mut widths: Vec<usize> = self.headers.iter().map(|h| Self::visible_width(h)).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                let w = Self::visible_width(cell);
                if w > widths[i] {
                    widths[i] = w;
                }
            }
        }
        widths
    }

    /// رندر جدول با کادر یونیکد.
    ///
    /// نمونهٔ خروجی:
    /// ```text
    /// ┌────┬──────────┐
    /// │ ID │ عنوان    │
    /// ├────┼──────────┤
    /// │  1 │ خرید نان │
    /// └────┴──────────┘
    /// ```
    pub fn render(&self) -> String {
        if self.headers.is_empty() {
            return String::new();
        }

        let widths = self.column_widths();
        let mut out = String::with_capacity((widths.iter().sum::<usize>() + 4 * widths.len()) * (self.rows.len() + 3));

        // تابع کمکی داخلی برای ساخت خط جداکنندهٔ افقی
        // (بستار/clousre که `widths` را قرض می‌گیرد)
        let separator = |left: char, mid: char, right: char| -> String {
            let mut s = String::new();
            s.push(left);
            for (i, w) in widths.iter().enumerate() {
                for _ in 0..(w + 2) {
                    s.push('─');
                }
                // کاراکتر گوشه/میانه — آخرین ستون، گوشهٔ راست را می‌گیرد
                if i + 1 == widths.len() {
                    s.push(right);
                } else {
                    s.push(mid);
                }
            }
            s.push('\n');
            s
        };

        // ── سرستون ──
        out.push_str(&separator('┌', '┬', '┐'));
        out.push('│');
        for (i, header) in self.headers.iter().enumerate() {
            let _ = write!(
                out,
                " {} │",
                Self::pad_cell(&color::bold(header), widths[i], self.right_align[i])
            );
        }
        out.push('\n');
        out.push_str(&separator('├', '┼', '┤'));

        // ── ردیف‌ها ──
        for row in &self.rows {
            out.push('│');
            for (i, cell) in row.iter().enumerate() {
                let _ = write!(out, " {} │", Self::pad_cell(cell, widths[i], self.right_align[i]));
            }
            out.push('\n');
        }
        out.push_str(&separator('└', '┴', '┘'));

        out
    }

    /// رندر ساده بدون کادر (برای خروجی کم‌حجم یا کپی‌کردن).
    pub fn render_plain(&self) -> String {
        let widths = self.column_widths();
        let mut out = String::new();

        let mut line = |cells: &[String]| {
            let parts: Vec<String> = cells
                .iter()
                .enumerate()
                .map(|(i, c)| Self::pad_cell(c, widths[i], self.right_align[i]))
                .collect();
            // `join` روی `Vec<String>` — ترفند تمیزکاری متن
            out.push_str(parts.join("  ").trim_end());
            out.push('\n');
        };

        line(&self.headers);
        for row in &self.rows {
            line(row);
        }
        out
    }
}

/// چاپ یک خط جداکنندهٔ افقی با عنوان (برای بخش‌های خروجی).
pub fn section(title: &str) -> String {
    let title = format!(" {title} ");
    let width = 60usize;
    let visible = color::strip_ansi(&title).chars().count();
    let left = if visible < width { (width - visible) / 2 } else { 0 };
    let right = width.saturating_sub(visible + left);
    format!(
        "{}{}{}\n",
        "─".repeat(left),
        color::bold(&title),
        "─".repeat(right)
    )
}

/// نوار پیشرفت متنی مثل `████░░░░ 50%`.
pub fn progress_bar(percent: f64, width: usize) -> String {
    // `clamp` جلوی خروجی خارج از بازه را می‌گیرد (NaN هم به ۰ نگاشت می‌شود در حالت عادی)
    let percent = if percent.is_nan() { 0.0 } else { percent.clamp(0.0, 100.0) };
    // تبدیل اعشاری به تعداد کاراکتر پرشده — با گرد کردن (نه برش کور)
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);

    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width - filled)
    );
    // رنگ بر اساس درصد
    let colored = if percent >= 100.0 {
        color::green(&bar)
    } else if percent >= 50.0 {
        color::cyan(&bar)
    } else if percent > 0.0 {
        color::yellow(&bar)
    } else {
        color::dim(&bar)
    };
    format!("{colored} {percent:>5.1}%")
}

/// کوتاه‌کردن متن بلند با «…» — با در نظر گرفتن کاراکتر (نه بایت).
///
/// 💣 اینجا یک باگ کلاسیک Rust را نشان می‌دهیم: برش `&s[..n]` روی بایت
/// انجام می‌شود و اگر وسط یک کاراکتر چندبایتی بیفتد، **پنیک** می‌کند!
/// راه درست: `chars().take(n)`.
pub fn truncate(s: &str, max_chars: usize) -> String {
    let plain = color::strip_ansi(s);
    if plain.chars().count() <= max_chars {
        return s.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut out: String = plain.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_and_padding() {
        assert_eq!(Table::visible_width("سلام"), 4);
        assert_eq!(Table::visible_width("\x1b[31mسلام\x1b[0m"), 4);
        // padding با متن فارسی و رنگی
        let padded = Table::pad_cell("\x1b[31mسلام\x1b[0m", 8, false);
        assert_eq!(Table::visible_width(&padded), 8);
    }

    #[test]
    fn render_has_borders_and_alignment() {
        let mut t = Table::new(["ID", "عنوان"]).right(0);
        t.push(["1", "خرید نان"]);
        t.push(["22", "کد"]);
        let out = t.render();
        assert!(out.contains('┌') && out.contains('┘') && out.contains('┼'));
        assert!(out.contains("خرید نان"));
        // ستون اول راست‌چین است → «1» باید با فاصله درست بیاید
        for line in out.lines().filter(|l| l.starts_with('│')) {
            assert_eq!(Table::visible_width(line), Table::visible_width(out.lines().next().unwrap()));
        }
    }

    #[test]
    fn short_rows_are_padded_and_long_rows_truncated() {
        let mut t = Table::new(["a", "b"]);
        t.push(["1"]);                 // کم
        t.push(["1", "2", "3"]);       // زیاد
        assert!(t.render().contains('│'));
        assert!(t.render_plain().lines().count() == 3);
    }

    #[test]
    fn truncate_is_char_safe() {
        // این خط اگر با برش بایتی نوشته شده بود، پنیک می‌کرد:
        assert_eq!(truncate("سلام دنیا", 3), "سل…");
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("", 0), "");
        assert_eq!(truncate("😀😀😀", 2), "😀…");
    }

    #[test]
    fn progress_bar_shapes() {
        let bar = progress_bar(50.0, 10);
        let plain = color::strip_ansi(&bar);
        assert!(plain.starts_with("█████░░░░░"), "{plain}");
        assert!(plain.contains("50.0%"));
        // حالت‌های مرزی
        assert!(color::strip_ansi(&progress_bar(0.0, 4)).starts_with("░░░░"));
        assert!(color::strip_ansi(&progress_bar(150.0, 4)).starts_with("████"));
        assert!(color::strip_ansi(&progress_bar(f64::NAN, 4)).starts_with("░░░░"));
    }

    #[test]
    fn section_has_expected_width() {
        let s = section("عنوان");
        let line = s.lines().next().unwrap();
        // ۶۰ کاراکتر دیده‌شدنی (در حالت بی‌رنگ)
        assert_eq!(Table::visible_width(line), 60);
    }
}
