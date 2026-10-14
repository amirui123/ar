//! # 🧩 JSON — پیاده‌سازی کامل، بدون هیچ وابستگی
//!
//! چرا خودمان JSON نوشتیم؟ چون این پروژه یک *آزمایشگاه آموزشی* است؛ در دنیای
//! واقعی از `serde` و `serde_json` استفاده کن (به `docs/17-ecosystem.md` نگاه کن).
//! ولی نوشتن یک پارسر، بهترین تمرین دنیا برای یادگیری Rust است:
//!
//! | مفهوم Rust | کجا در این فایل |
//! |-----------|------------------|
//! | `enum` با داده (ADT) | [`Json`] |
//! | بازگشت (recursion) | `Parser::parse_value` |
//! | کار با بایت‌ها و `&str` | `Parser` روی `&[u8]` |
//! | `Result` و `?` | همه‌جا |
//! | `Display` برای سریالایز | `impl Display for Json` |
//! | پیاده‌سازی `From` | پایین فایل |
//! | تست واحد و «رِند تست» (property-ish) | `mod tests` |
//!
//! 🔍 **ترفند حرفه‌ای:** پارسر ما روی **بایت** کار می‌کند (`&[u8]`)، نه `char`.
//! JSON یک قالب متنی ASCII-محور است؛ بایت‌به‌بایت جلو رفتن سریع‌تر از
//! `chars().nth()` است و از پنیکِ «برش وسط کاراکتر» هم جلوگیری می‌کند.

use std::fmt::{self, Write as _};

// ═══════════════════════════════════════════════════════════════════════════
//  ۱) نوع داده — «درخت JSON»
// ═══════════════════════════════════════════════════════════════════════════
/// یک مقدار JSON.
///
/// نکتهٔ طراحی: شیء (object) را با `Vec<(String, Json)>` نگه می‌داریم نه `HashMap`،
/// چون **ترتیب کلیدها** باید حفظ شود (خروجی ما باید پایدار/قابل diff باشد) و
/// تعداد کلیدها کم است. اگر جست‌وجوی کلید مهم‌تر از ترتیب بود، `BTreeMap` می‌گرفتیم؛
/// اگر سرعت مهم‌تر بود، `HashMap`.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// `null`
    Null,
    /// `true` / `false`
    Bool(bool),
    /// عدد صحیح (بدون نقطه و نمایندهٔ علمی)
    Int(i64),
    /// عدد اعشاری
    Float(f64),
    /// رشته
    Str(String),
    /// آرایه
    Arr(Vec<Json>),
    /// شیء (نگاشت کلید → مقدار، به ترتیب درج)
    Obj(Vec<(String, Json)>),
}

impl Json {
    // ── سازنده‌های کمکی ────────────────────────────────────────────────────
    /// ساخت شیء از یک «لیست جفت‌ها» — با این امضا هم آرایه و هم `Vec` می‌پذیرد.
    ///
    /// 🔑 `K: Into<String>` یعنی هم `&str` هم `String` قبول است؛
    /// و `I: IntoIterator` یعنی هم `vec![...]` هم هر چیز پیمایش‌پذیر دیگر.
    pub fn obj<K, I>(entries: I) -> Json
    where
        K: Into<String>,
        I: IntoIterator<Item = (K, Json)>,
    {
        Json::Obj(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// آیا `null` است؟
    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }

    // ── دسترسی نوع‌دار (typed access) ───────────────────────────────────────
    /// تلاش برای دیدن به‌عنوان رشته.
    pub fn as_str(&self) -> Option<&str> {
        // `if let` + الگوی «binding با ref» → هیچ کپی‌ای رخ نمی‌دهد
        match self {
            Json::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// دیدن به‌عنوان `bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b), // bool کپی‌شدنی است، پس deref و کپی
            _ => None,
        }
    }

    /// دیدن به‌عنوان عدد صحیح (`Int`؛ و `Float` فقط اگر اعشاری نداشته باشد).
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Int(n) => Some(*n),
            // ترفند: 3.0 → 3 ولی 3.5 → None
            Json::Float(x) if x.fract() == 0.0 && x.is_finite() && *x >= i64::MIN as f64 && *x <= i64::MAX as f64 => {
                Some(*x as i64)
            }
            _ => None,
        }
    }

    /// دیدن به‌عنوان عدد اعشاری (از هر دو حالت عددی).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Int(n) => Some(*n as f64),
            Json::Float(x) => Some(*x),
            _ => None,
        }
    }

    /// دیدن به‌عنوان آرایه.
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(v) => Some(v),
            _ => None,
        }
    }

    /// دیدن به‌عنوان لیست کلید-مقدار شیء.
    pub fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Obj(m) => Some(m),
            _ => None,
        }
    }

    // ── دسترسی به کلیدهای شیء ──────────────────────────────────────────────
    /// مقدار یک کلید (اگر شیء بود و کلید وجود داشت).
    pub fn get(&self, key: &str) -> Option<&Json> {
        self.as_object()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// رشتهٔ یک کلید، با پیش‌فرض در صورت نبودن/غلط بودن نوع.
    ///
    /// این «ترفندِ تحمل‌پذیری» (tolerance) برای خواندن فایل‌های قدیمی مفید است.
    pub fn get_str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(Json::as_str).unwrap_or(default)
    }

    /// عدد صحیح یک کلید.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(Json::as_i64)
    }

    /// پرچم (bool) یک کلید.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(Json::as_bool)
    }

    /// آرایهٔ یک کلید.
    pub fn get_array(&self, key: &str) -> Option<&[Json]> {
        self.get(key).and_then(Json::as_array)
    }

    /// طول (برای آرایه/شیء/رشته). برای بقیه انواع `None`.
    pub fn len(&self) -> Option<usize> {
        match self {
            Json::Arr(v) => Some(v.len()),
            Json::Obj(m) => Some(m.len()),
            Json::Str(s) => Some(s.chars().count()), // ترفند: طول *کاراکتر* نه بایت!
            _ => None,
        }
    }

    /// خالی بودن (با احترام به تعریف `len`).
    pub fn is_empty(&self) -> bool {
        self.len() == Some(0)
    }

    /// نوع مقدار به‌صورت متن فارسی (برای پیام خطا).
    pub fn type_name(&self) -> &'static str {
        match self {
            Json::Null => "null",
            Json::Bool(_) => "منطقی",
            Json::Int(_) => "عدد صحیح",
            Json::Float(_) => "عدد اعشاری",
            Json::Str(_) => "رشته",
            Json::Arr(_) => "آرایه",
            Json::Obj(_) => "شیء",
        }
    }

    // ── خروجی زیبا (pretty print) ──────────────────────────────────────────
    /// نسخهٔ خوانا با تودرتویی.
    pub fn to_pretty(&self, indent: usize) -> String {
        let mut out = String::with_capacity(256);
        self.write_pretty(&mut out, 0, indent);
        out
    }

    /// نوشتن بازگشتی نسخهٔ زیبا.
    ///
    /// نکته: `&mut String` را پاس می‌دهیم (الگوی «output parameter درونی»)،
    /// چون هر بار برگرداندن `String` جدید باعث تخصیص‌های اضافی می‌شود.
    fn write_pretty(&self, out: &mut String, depth: usize, indent: usize) {
        match self {
            Json::Arr(items) if !items.is_empty() => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    push_indent(out, depth + 1, indent);
                    item.write_pretty(out, depth + 1, indent);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                push_indent(out, depth, indent);
                out.push(']');
            }
            Json::Obj(map) if !map.is_empty() => {
                out.push_str("{\n");
                for (i, (k, v)) in map.iter().enumerate() {
                    push_indent(out, depth + 1, indent);
                    // کلید هم باید escape شود (مثلاً اگر نقل‌قول داشت)
                    let _ = write!(out, "{}: ", escape_str(k));
                    v.write_pretty(out, depth + 1, indent);
                    if i + 1 < map.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                push_indent(out, depth, indent);
                out.push('}');
            }
            // آرایه/شیء خالی یا انواع ساده → تک‌خطی
            other => {
                let _ = write!(out, "{other}");
            }
        }
    }
}

/// درج فاصلهٔ تودرتویی.
fn push_indent(out: &mut String, depth: usize, indent: usize) {
    for _ in 0..depth * indent {
        out.push(' ');
    }
}

/// فرار دادن کاراکترهای خاص رشته (quote، backslash، کنترلی‌ها) طبق RFC 8259.
pub fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                // بقیهٔ کاراکترهای کنترلی با \uXXXX
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── خروجی فشرده با Display ──────────────────────────────────────────────────
impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => f.write_str("null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Int(n) => write!(f, "{n}"),
            Json::Float(x) => {
                // ترفند: عدد اعشاری بدون بخش اعشاری را «3.0» بنویس تا JSON معتبر باشد
                // (وگرنه بعضی زبان‌ها آن را صحیح می‌فهمند).
                if x.is_finite() && x.fract() == 0.0 {
                    write!(f, "{x:.1}")
                } else {
                    write!(f, "{x}")
                }
            }
            Json::Str(s) => f.write_str(&escape_str(s)),
            Json::Arr(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{item}")?; // بازگشت به Display خودمان
                }
                f.write_str("]")
            }
            Json::Obj(map) => {
                f.write_str("{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    f.write_str(&escape_str(k))?;
                    f.write_str(":")?;
                    write!(f, "{v}")?;
                }
                f.write_str("}")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۲) تبدیل‌های راحت (From / Into)
// ═══════════════════════════════════════════════════════════════════════════
//  با پیاده‌سازی `From<X> for Json`، هم `Json::from(x)` و هم `x.into()` کار می‌کند
//  و هم پارامترهای `impl Into<Json>` راحت می‌شوند. این همان «قرارداد تبدیل» زبان است.

impl From<bool> for Json {
    fn from(v: bool) -> Self {
        Json::Bool(v)
    }
}
impl From<i64> for Json {
    fn from(v: i64) -> Self {
        Json::Int(v)
    }
}
impl From<i32> for Json {
    fn from(v: i32) -> Self {
        Json::Int(v as i64) // تبدیل بدون ضرر (i32 داخل i64 جا می‌شود)
    }
}
impl From<u32> for Json {
    fn from(v: u32) -> Self {
        Json::Int(v as i64)
    }
}
impl From<usize> for Json {
    fn from(v: usize) -> Self {
        Json::Int(v as i64)
    }
}
impl From<u64> for Json {
    fn from(v: u64) -> Self {
        // ترفند: u64 ممکن است از i64 بزرگ‌تر باشد → در آن حالت اعشاری ذخیره می‌کنیم
        if v <= i64::MAX as u64 {
            Json::Int(v as i64)
        } else {
            Json::Float(v as f64)
        }
    }
}
impl From<f64> for Json {
    fn from(v: f64) -> Self {
        Json::Float(v)
    }
}
impl From<&str> for Json {
    fn from(v: &str) -> Self {
        Json::Str(v.to_string())
    }
}
impl From<String> for Json {
    fn from(v: String) -> Self {
        Json::Str(v)
    }
}
impl From<Vec<Json>> for Json {
    fn from(v: Vec<Json>) -> Self {
        Json::Arr(v)
    }
}
impl<T: Into<Json>> From<Option<T>> for Json {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(x) => x.into(),
            None => Json::Null,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۳) خطای پارس — با موقعیت دقیق
// ═══════════════════════════════════════════════════════════════════════════
/// خطای پارس JSON، همراه با «کجا» (خط، ستون و بایت).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// پیام فارسی قابل‌فهم
    pub message: String,
    /// آفست بایتی از ابتدای ورودی
    pub offset: usize,
    /// شمارهٔ خط (از ۱)
    pub line: usize,
    /// شمارهٔ ستون (از ۱، بر اساس کاراکتر)
    pub column: usize,
}

impl JsonError {
    /// ساخت خطا در آفست مشخص. خط/ستون از خود متن حساب می‌شود.
    ///
    /// 🔑 **ترفند:** محاسبهٔ خط/ستون به این شکل ساده است، ولی هزینهٔ `O(n)` دارد.
    /// چون فقط *وقتی خطا رخ می‌دهد* صدا زده می‌شود، این هزینه هیچ اهمیتی ندارد —
    /// به این «بهینه‌سازی مسیر خوش‌بینانه» (optimistic fast path) می‌گویند.
    pub fn at(src: &str, offset: usize, message: impl Into<String>) -> Self {
        let offset = offset.min(src.len());
        let before = &src[..offset];
        let line = before.matches('\n').count() + 1;
        // ستون = تعداد کاراکترهای بعد از آخرین \n
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = src[line_start..offset].chars().count() + 1;
        JsonError { message: message.into(), offset, line, column }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "خطای JSON در خط {}، ستون {} (بایت {}): {}",
            self.line, self.column, self.offset, self.message
        )
    }
}

impl std::error::Error for JsonError {}

/// نام مستعار برای خوانایی
type ParseResult<T> = std::result::Result<T, JsonError>;

// ═══════════════════════════════════════════════════════════════════════════
//  ۴) پارسر — «کاهش بازگشتی» (recursive descent)
// ═══════════════════════════════════════════════════════════════════════════
/// پارسر JSON.
///
/// ساختار: یک اشاره‌گر موقعیت (`pos`) روی بایت‌های ورودی + متدهایی که
/// هر یک تکه‌ای از گرامر را می‌شناسند.
///
/// ⏳ **عمر (lifetime):** `Parser<'a>` یعنی «این پارسر *قرض‌گیرندهٔ* متنی است
/// که حداقل `'a` عمر دارد». ما هیچ‌جا کپی از کل ورودی نمی‌گیریم.
struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser { src, bytes: src.as_bytes(), pos: 0 }
    }

    /// بایت فعلی (بدون جلو رفتن)
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// بایت فعلی و یک قدم جلو
    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    /// پرش از فاصله‌های مجاز (فقط این چهار: space, tab, LF, CR)
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    /// ساخت خطا (به‌صورت مقدار، نه Result) — برای استفاده در closureهای `ok_or_else`
    fn err(&self, msg: impl Into<String>) -> JsonError {
        JsonError::at(self.src, self.pos, msg)
    }

    /// `Result::Err` با پیام مشخص
    fn fail<T>(&self, msg: impl Into<String>) -> ParseResult<T> {
        Err(self.err(msg))
    }

    /// مصرف یک بایت مشخص یا خطا
    fn expect(&mut self, want: u8) -> ParseResult<()> {
        match self.peek() {
            Some(b) if b == want => {
                self.pos += 1;
                Ok(())
            }
            Some(b) => self.fail(format!("انتظار «{}» بود، ولی «{}» دیدم", want as char, b as char)),
            None => self.fail(format!("پایان ناگهانی ورودی؛ انتظار «{}» بود", want as char)),
        }
    }

    /// تطبیق یک کلمهٔ کلیدی ثابت مثل `true`
    fn keyword(&mut self, kw: &str) -> ParseResult<()> {
        if self.src[self.pos..].starts_with(kw) {
            self.pos += kw.len();
            Ok(())
        } else {
            self.fail(format!("انتظار «{kw}» بود"))
        }
    }

    /// مقدار بعدی را پارس کن (نقطهٔ ورود بازگشت).
    fn parse_value(&mut self) -> ParseResult<Json> {
        self.skip_ws();
        match self.peek() {
            None => self.fail("پایان ورودی؛ انتظار یک مقدار JSON بود"),
            Some(b'n') => {
                self.keyword("null")?;
                Ok(Json::Null)
            }
            Some(b't') => {
                self.keyword("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.keyword("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'"') => Ok(Json::Str(self.parse_string()?)),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(c) => self.fail(format!("کاراکتر غیرمنتظره «{}» (بایت {:#04x})", c as char, c)),
        }
    }

    /// پارس رشتهٔ دو-گیومه‌ای با escape ها.
    fn parse_string(&mut self) -> ParseResult<String> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();

        loop {
            let b = match self.bump() {
                Some(b) => b,
                None => return self.fail("رشته بسته نشد (گیومهٔ پایانی پیدا نشد)"),
            };

            match b {
                b'"' => break, // پایان رشته
                b'\\' => {
                    // فرار (escape)
                    let esc = match self.bump() {
                        Some(e) => e,
                        None => return self.fail("رشته بعد از «\\» تمام شد"),
                    };
                    match esc {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0c),
                        b'u' => {
                            let ch = self.parse_unicode_escape()?;
                            // کاراکتر را به بایت‌های UTF-8 تبدیل و اضافه کن
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => return self.fail(format!("فرار ناشناخته: \\{}", other as char)),
                    }
                }
                // کاراکترهای کنترلی خام در JSON مجاز نیستند (باید escape شوند)
                0x00..=0x1f => {
                    return self.fail(format!("کاراکتر کنترلی خام (بایت {b:#04x}) در رشته مجاز نیست"))
                }
                other => out.push(other),
            }
        }

        // اعتبارسنجی UTF-8 — چون بایت‌به‌بایت جلو رفتیم، این چک لازم است.
        String::from_utf8(out)
            .map_err(|e| self.err(format!("بایت‌های UTF-8 نامعتبر در رشته: {e}")))
    }

    /// چهار رقم هگز (برای `\uXXXX`)
    fn hex4(&mut self) -> ParseResult<u32> {
        let mut value = 0u32;
        for i in 0..4 {
            let b = self.bump().ok_or_else(|| self.err("کد «\\u» به ۴ رقم هگز نیاز دارد"))?;
            let digit = (b as char)
                .to_digit(16)
                .ok_or_else(|| self.err(format!("رقم هگز نامعتبر «{}» در جایگاه {i}", b as char)))?;
            value = value * 16 + digit;
        }
        Ok(value)
    }

    /// یک escape یونیکد را به کاراکتر تبدیل کن (شامل جفت‌های جانشین).
    ///
    /// 😈 **نکتهٔ شیطانی:** در JSON، کاراکترهای بیرون از «صفحهٔ پایه»
    /// (مثل ایموجی 😀 = U+1F600) به شکل *دو* escape پشت‌سرهم می‌آیند
    /// (surrogate pair) — چون JSON از UTF-16 ارث‌بری کرده. اگر این را
    /// پیاده نکنی، ورودی‌های واقعی دنیا (که توسط JavaScript ساخته شده‌اند)
    /// خراب می‌شوند.
    fn parse_unicode_escape(&mut self) -> ParseResult<char> {
        let hi = self.hex4()?;

        let code_point = if (0xD800..0xDC00).contains(&hi) {
            // «جانشین بالا» → باید یک «جانشین پایین» بعدش باشد
            if self.peek() == Some(b'\\') && self.bytes.get(self.pos + 1) == Some(&b'u') {
                self.pos += 2; // از \\u بگذر
                let lo = self.hex4()?;
                if !(0xDC00..0xE000).contains(&lo) {
                    return self.fail("نیمهٔ دوم جفت جانشین در بازهٔ درست نیست");
                }
                0x1_0000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
            } else {
                return self.fail("جانشین بالا بدون جفت پایین‌اش آمد");
            }
        } else if (0xDC00..0xE000).contains(&hi) {
            return self.fail("جانشین پایین تنها (بدون جفت بالا) مجاز نیست");
        } else {
            hi
        };

        char::from_u32(code_point).ok_or_else(|| self.err(format!("کد یونیکد نامعتبر: U+{code_point:04X}")))
    }

    /// پارس عدد (صحیح یا اعشاری، با نمایندهٔ علمی).
    fn parse_number(&mut self) -> ParseResult<Json> {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.pos += 1;
        }

        // بخش صحیح: یا «0» تنها، یا رقم غیرصفر و بعد هر تعداد رقم
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return self.fail("عدد JSON نامعتبر: رقم انتظار می‌رفت"),
        }

        let mut is_float = false;

        // بخش اعشاری
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return self.fail("بعد از «.» باید حداقل یک رقم بیاید");
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        // نمایندهٔ علمی (e/E)
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return self.fail("در نمایندهٔ علمی باید رقم بیاید");
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        // 🔑 ترفند: چون `src: &'a str` است، این برش **هیچ کپی‌ای** نمی‌سازد.
        let text = &self.src[start..self.pos];

        if !is_float {
            // اعداد صحیحی که در i64 جا نمی‌شوند → به اعشاری تنزل می‌کنیم
            if let Ok(n) = text.parse::<i64>() {
                return Ok(Json::Int(n));
            }
        }

        text.parse::<f64>()
            .map(Json::Float)
            .map_err(|_| self.err(format!("عدد نامعتبر: «{text}»")))
    }

    /// پارس آرایه: `[ v, v, ... ]`
    fn parse_array(&mut self) -> ParseResult<Json> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();

        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Arr(items)); // آرایهٔ خالی
        }

        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.bump() {
                Some(b',') => {
                    self.skip_ws();
                    continue; // عنصر بعدی
                }
                Some(b']') => break,
                Some(c) => {
                    return self.fail(format!("انتظار «,» یا «]» بود، ولی «{}» دیدم", c as char));
                }
                None => return self.fail("آرایه بسته نشد (بدون «]»)"),
            }
        }

        Ok(Json::Arr(items))
    }

    /// پارس شیء: `{ "k": v, ... }`
    fn parse_object(&mut self) -> ParseResult<Json> {
        self.expect(b'{')?;
        let mut map: Vec<(String, Json)> = Vec::new();
        self.skip_ws();

        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Obj(map)); // شیء خالی
        }

        loop {
            self.skip_ws();
            let key = self.parse_string()?; // کلید باید رشته باشد
            self.skip_ws();
            self.expect(b':')?;
            let value = self.parse_value()?;

            // ترفند: کلید تکراری → «آخری برنده است» (رفتار مرورگرها هم همین است)
            if let Some(slot) = map.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = value;
            } else {
                map.push((key, value));
            }

            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b'}') => break,
                Some(c) => return self.fail(format!("انتظار «,» یا «}}» بود، ولی «{}» دیدم", c as char)),
                None => return self.fail("شیء بسته نشد (بدون «}»)"),
            }
        }

        Ok(Json::Obj(map))
    }
}

/// پارس یک متن JSON.
///
/// # خطاها
/// اگر متن JSON معتبر نباشد، [`JsonError`] با موقعیت دقیق برمی‌گردد.
///
/// # مثال
/// ```
/// use ar::json::{parse, Json};
///
/// let v = parse(r#"{"name": "علی", "age": 30, "tags": ["a"]}"#).unwrap();
/// assert_eq!(v.get("name").and_then(Json::as_str), Some("علی"));
/// assert_eq!(v.get_i64("age"), Some(30));
/// assert_eq!(v.get_array("tags").map(<[Json]>::len), Some(1));
/// ```
pub fn parse(src: &str) -> ParseResult<Json> {
    let mut p = Parser::new(src);
    let value = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        // داده‌های اضافه بعد از مقدار → خطای سخت‌گیرانه (مثل اکثر پارسرهای جدی)
        return p.fail("داده‌های اضافی بعد از پایان مقدار JSON");
    }
    Ok(value)
}

// ═══════════════════════════════════════════════════════════════════════════
//  ۵) تست‌ها
// ═══════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    /// کمکی: پارس + بازگرداندن خطا به رشته برای assert
    fn p(s: &str) -> ParseResult<Json> {
        parse(s)
    }

    #[test]
    fn scalars() {
        assert_eq!(p("null").unwrap(), Json::Null);
        assert_eq!(p("true").unwrap(), Json::Bool(true));
        assert_eq!(p("false").unwrap(), Json::Bool(false));
        assert_eq!(p(" 42 ").unwrap(), Json::Int(42));
        assert_eq!(p("-17").unwrap(), Json::Int(-17));
        assert_eq!(p("3.5").unwrap(), Json::Float(3.5));
        assert_eq!(p("1e3").unwrap(), Json::Float(1000.0));
        assert_eq!(p("0").unwrap(), Json::Int(0));
    }

    #[test]
    fn strings_and_escapes() {
        assert_eq!(p(r#""سلام""#).unwrap(), Json::Str("سلام".into()));
        assert_eq!(p(r#""a\nb""#).unwrap(), Json::Str("a\nb".into()));
        assert_eq!(p(r#""\u0633\u0644\u0627\u0645""#).unwrap(), Json::Str("سلام".into()));
        assert_eq!(p(r#""q\"q""#).unwrap(), Json::Str(r#"q"q"#.into()));
        // ایموجی با جفت جانشین (همان چیزی که JSON.stringify تولید می‌کند)
        assert_eq!(p(r#""\ud83d\ude00""#).unwrap(), Json::Str("😀".into()));
        // رشتهٔ UTF-8 مستقیم باید همان‌طور بماند
        assert_eq!(p("\"😀\"").unwrap(), Json::Str("😀".into()));
    }

    #[test]
    fn nested_structures() {
        let src = r#"{"a":[1,2,{"b":null}],"c":{}}"#;
        let v = p(src).unwrap();
        assert_eq!(v.get("a").unwrap().as_array().unwrap().len(), 3);
        assert!(v.get("c").unwrap().is_empty());
        assert!(v.get("missing").is_none());
    }

    #[test]
    fn duplicate_keys_last_wins() {
        let v = p(r#"{"x":1,"x":2}"#).unwrap();
        assert_eq!(v.get_i64("x"), Some(2));
        assert_eq!(v.as_object().unwrap().len(), 1);
    }

    #[test]
    fn errors_have_position() {
        let e = parse("{\n  \"a\": tru\n}").unwrap_err();
        assert_eq!(e.line, 2);
        assert!(e.column > 1, "ستون باید محاسبه شده باشد: {e}");
        // توجه: ارقام در خروجی `{}` لاتین‌اند (۲ فارسی نیست) — همین یک تلهٔ ریز است!
        assert!(e.to_string().contains("خط 2"), "{e}");
    }

    #[test]
    fn rejects_trailing_garbage_and_bad_syntax() {
        assert!(parse("1 2").is_err());
        assert!(parse("[1,]").is_err());        // ویرگول پایانی مجاز نیست
        assert!(parse("{\"a\":1,}").is_err());
        assert!(parse("01").is_err());          // صفر پیشرو
        assert!(parse("\"unterminated").is_err());
        assert!(parse("").is_err());
        assert!(parse("nul").is_err());
        assert!(parse("\"\\ud83d\"").is_err()); // جانشین تنها
    }

    #[test]
    fn round_trip_compact_and_pretty() {
        let v = Json::obj(vec![
            ("name", Json::from("علی")),
            ("n", Json::from(3i64)),
            ("ok", Json::from(true)),
            ("nothing", Json::Null),
            ("list", Json::Arr(vec![Json::from(1i64), Json::from(2i64)])),
        ]);

        let compact = v.to_string();
        // ✅ مهم‌ترین تست: خروجی ما باید دوباره قابل پارس باشد (round-trip)
        assert_eq!(parse(&compact).unwrap(), v, "compact: {compact}");

        let pretty = v.to_pretty(2);
        assert_eq!(parse(&pretty).unwrap(), v, "pretty:\n{pretty}");
        assert!(pretty.contains('\n'));
        // ترتیب کلیدها باید حفظ شود
        assert_eq!(
            v.as_object().unwrap().iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            vec!["name", "n", "ok", "nothing", "list"]
        );
    }

    #[test]
    fn float_formatting_is_valid_json() {
        let v = Json::Float(3.0);
        assert_eq!(v.to_string(), "3.0"); // نه «3» — تا JSON معتبر بماند
        assert!(parse(&v.to_string()).is_ok());
    }

    #[test]
    fn deep_nesting_parses() {
        // ۱۰۰ سطح تودرتویی — بازگشت باید سالم کار کند
        let src = "[".repeat(100) + &"]".repeat(100);
        assert!(parse(&src).is_ok());
    }
}
