# 🦀 ar — آزمایشگاه کامل Rust در قالب یک برنامهٔ واقعی

> یک مدیر تسک (task manager) کامل با خط فرمان، سرور وب، ذخیره‌سازی JSON،
> آمار و جدول‌سازی — **با صفر وابستگی بیرونی**، و با کامنتِ فارسی خط‌به‌خط.
>
> هدف: به‌جای خواندن تئوری، **کدِ واقعیِ کارکننده** را ببینی و هر ترفندش را یاد بگیری.

<p align="center">
  <b>Rust 1.85+ • edition 2024 • ⚡ بدون هیچ crate بیرونی (فقط <code>std</code>)</b>
</p>

---

## ⚡ در ۶۰ ثانیه

```bash
cargo run -- seed                                  # ۵ تسک نمونه
cargo run -- list                                  # جدول رنگی تسک‌ها
cargo run -- add "یاد گرفتن Rust" --priority high --tags rust,یادگیری
cargo run -- stats                                 # آمار و نمودار متنی
cargo run -- serve --port 8080                     # سرور وب + API جیسون
cargo run -- demo all                              # 🎓 اجرای دموی همهٔ مفاهیم زبان
cargo run -- demo ownership                        # فقط یک مفهوم
```

**هیچ اینترنتی لازم نیست.** سرور وب، پارسر JSON، جدول‌ساز، پارسر CLI و
اجراکنندهٔ تردها همه از صفر با `std` نوشته شده‌اند.

---

## 🗺️ نقشهٔ پروژه — چه چیزی را کجا یاد می‌گیری

هر ماژول یک «درس» است. ترتیب پیشنهادی مطالعه همان ترتیب جدول است:

| ترتیب | فایل | مفاهیم کلیدی |
|:---:|---|---|
| ۱ | [`src/model.rs`](src/model.rs) | `struct`، `enum`، `derive`، **newtype**، `Display`، `FromStr`، `Ordering` |
| ۲ | [`src/errors.rs`](src/errors.rs) | `Result`، عملگر `?`، `From`، زنجیرهٔ علت خطا، `#[macro_export]` |
| ۳ | [`src/json.rs`](src/json.rs) | enum با داده، بازگشت (recursion)، پارسر بایت‌به‌بایت، `Display`، تست round-trip |
| ۴ | [`src/store.rs`](src/store.rs) | **صفت (trait)**، `Box<dyn>`، `RefCell`، نوشتن **اتمیک** فایل، نسخه‌دار کردن قالب |
| ۵ | [`src/app.rs`](src/app.rs) | **ایتراتور**، **بستار**، `HashMap`/`BTreeMap`، الگوی `entry`، `Default` |
| ۶ | [`src/cli.rs`](src/cli.rs) | `env::args`، `Peekable`، ژنریک با `FromStr`، قواعد پارس فلگ |
| ۷ | [`src/table.rs`](src/table.rs) | `write!`، عرض یونیکد، **پاک‌کردن کدهای ANSI**، برش امن رشته |
| ۸ | [`src/color.rs`](src/color.rs) | ANSI، `static` + **اتمیک**، احترام به `NO_COLOR`، `IsTerminal` |
| ۹ | [`src/web.rs`](src/web.rs) | `TcpListener`، پارس HTTP دستی، `Arc<Mutex<T>>`، `include_str!` |
| ۱۰ | [`src/report.rs`](src/report.rs) | تولید **HTML ایستا با Rust**: فرار متن (ضد XSS)، تاریخ بدون crate (الگوریتم civil_from_days)، `data-*` + فیلتر سمت مرورگر |
| ۱۱ | [`src/demos/`](src/demos) | ۱۵ دموی مستقل: مالکیت، عمر، صفت‌ها، unsafe، ماکرو، ایتراتور و … |

📚 و پوشهٔ [`docs/`](docs/README.md) یک **مرجع فارسی کامل** است: ۱۸ سند، از سینتکس تا ترفندهای حرفه‌ای.
از [`docs/README.md`](docs/README.md) (فهرست + مسیرهای مطالعه) یا
[`docs/00-roadmap.md`](docs/00-roadmap.md) (برنامهٔ ۶ هفته‌ای) شروع کن.

---

## 🎓 دموها — مفاهیم زبان، اجراشده جلوی چشمت

```bash
cargo run -- demo            # فهرست موضوع‌ها
cargo run -- demo all        # همه (چند ثانیه طول می‌کشد)
```

| موضوع | چه می‌بینی |
|---|---|
| `ownership` | انتقال/کپی، قرض ثابت و تغییرپذیر، NLL، ترتیب `Drop` |
| `patterns` | `match`، تفکیک ساختار، `guard`، بازه، `@`، الگوهای برش، `let-else` |
| `strings` | `String` vs `&str`، دام بایت/کاراکتر، `Cow`، `from_utf8` |
| `numeric` | سرریز و چهار راهش، `as` vs `try_into`، دام‌های `f64`، `const fn` |
| `collections` | `Vec`/`HashMap`/`BTreeMap`/`VecDeque`/`HashSet`/`BinaryHeap` + `entry` |
| `iterator` | تنبلی، آداپتورها، مصرف‌کننده‌ها، **ایتراتور دلخواه**، ایتراتور نامتناهی |
| `generics` | قیدها، `where`، enum ژنریک، نوع وابسته، سربارگذاری اپراتور، const generics |
| `traits_adv` | متد پیش‌فرض، سوپرتریت، blanket impl، **صفت مهرشده**، `dyn` vs استاتیک |
| `closures` | `Fn`/`FnMut`/`FnOnce`، `move`، برگرداندن بستار، حافظه‌سازی |
| `smart_pointers` | `Box`/`Rc`/`RefCell`/`Cell`/`Weak`/`Arc<Mutex>`, **چرخهٔ حافظه** |
| `concurrency` | ترد، کانال، اتمیک، `thread::scope`، **پیاده‌سازی استخر ترد** |
| `lifetimes` | سه قاعدهٔ حذف عمر، `'static`، ساختار با مرجع، راه‌های فرار |
| `macros` | `macro_rules!`، تکرار، جویدن TT، اطلاعات زمان کامپایل، تولید تست |
| `unsafe_ffi` | اشاره‌گر خام، `extern "C"`، `#[repr(C)]`، قرارداد `# Safety`، `into_raw` |
| `errors` | `Result`، ترکیب‌کننده‌ها، خطای سفارشی + `source()`، پنیک کِی درست است |

---

## 🧰 امکانات برنامه

### خط فرمان

```bash
ar add <عنوان> [--priority low|normal|high|urgent] [--tags a,b,c]
ar list [--status all|pending|done] [--tag T] [--search Q] [--sort id|urgency|priority|newest|oldest|title] [--limit N] [--reverse] [--json]
ar done <#>… | ar undo <#>… | ar rm <#>… [--force] | ar purge
ar stats [--json] | ar export | ar seed | ar html [--out report.html]
ar serve [--port 8080] [--host 0.0.0.0]
bash scripts/run-web.sh   # همین سرور + چک/ساخت خودکار تول‌چین، باینری و داده
ar demo <موضوع>
```

فلگ‌های عمومی: `--data <مسیر>`، `--memory`، `--color`/`--no-color`، `-h`.

### سرور وب

```bash
ar serve --port 8080
```

| مسیر | متد | کار |
|---|---|---|
| `/` | GET | رابط وب (HTML داخل باینری حک شده) |
| `/api/tasks` | GET | فهرست (با `?status=&tag=&q=&sort=&limit=`) |
| `/api/tasks` | POST | ساخت: `{"title":"…","priority":"high","tags":["a","b"]}` |
| `/api/tasks/:id/toggle` | POST | تغییر وضعیت انجام |
| `/api/tasks/:id` | DELETE | حذف |
| `/api/stats` | GET | آمار |
| `/api/purge` | POST | پاک‌سازی انجام‌شده‌ها |
| `/health` | GET | بررسی سلامت |

همهٔ خروجی‌ها UTF-8 و `charset=utf-8` هستند (فارسی درست منتقل می‌شود).

---

## 🚀 نصب و اجرا

```bash
git clone https://github.com/amirui123/ar
cd ar
cargo run -- seed
```

نیازمندی: **Rust 1.85 یا بالاتر** (برای edition 2024).
اگر Rust نداری: [docs/01-install-and-cargo.md](docs/01-install-and-cargo.md) — شامل ترفندِ
نصب در محیط‌هایی که به سرورهای رسمی Rust دسترسی ندارند (با npm!)

```bash
cargo test          # ۱۴۰ تست واحد + ۹ تست یکپارچه + doctestها
cargo run --example quickstart   # ۲۰ خط، کل API کتابخانه
cargo run --example report       # گزارش خوانا از دادهٔ برنامه
cargo build --release
cargo run --release -- list
```

---

## 💡 ۱۲ ترفندی که در این پروژه یاد می‌گیری

۱. **`?` + `From`** = خطاها بدون یک خط `map_err` اضافی تبدیل می‌شوند. → `errors.rs`
۲. **عملگر `?` روی `Option`** = کد زنجیره‌ای تمیز برای «ممکن است نباشد». → `app.rs`
۳. **نوشتن اتمیک فایل** (`tmp` + `rename`) = حتی با قطع برق، فایل خراب نمی‌شود. → `store.rs`
۴. **`Rc<RefCell<T>>`** = گراف قابل‌تغییر؛ و **`Weak`** برای شکستن چرخهٔ حافظه. → `smart_pointers.rs`
۵. **`entry` API** = یک جست‌وجو برای درج *و* افزایش شمارنده (بدون دو بار هش‌کردن). → `collections.rs`
۶. **`collect::<Result<Vec<_>>>()`** = «یا همه موفق، یا اولین خطا». → `iterator.rs`
۷. **`impl Fn` + `move`** = برگرداندن بستار از تابع بدون دردسر عمر. → `closures.rs`
۸. **`&TcpStream` هم `Read` است هم `Write`** = یک سوکت، دو کار، بدون `clone`. → `web.rs`
۹. **`#[repr(transparent)]`** = newtype با **صفر هزینهٔ اجرا**. → `model.rs`
۱۰. **`with_capacity`** = حذف تخصیص‌های تکراری در حلقه‌ها. → `collections.rs`
۱۱. **`debug_assert!` / `const { assert!(…) }`** = بررسی‌هایی که در release حذف می‌شوند. → `macros_demo.rs`
۱۲. **`std::thread::scope`** = ترد موازی با دادهٔ محلی، **بدون `Arc` و بدون `move`**. → `concurrency.rs`

و ۸۸ ترفند دیگر، با کد و دلیل، در [docs/12-tricks.md](docs/12-tricks.md).
دام‌های معروف هم در [docs/13-pitfalls.md](docs/13-pitfalls.md).

---

## 🧠 فهرست مرجع فارسی (`docs/`)

> 🧭 فهرست کامل با «مسیرهای مطالعهٔ» پیشنهادی: [`docs/README.md`](docs/README.md)

| فایل | موضوع |
|---|---|
| [00-roadmap.md](docs/00-roadmap.md) | نقشهٔ راه، چطور یاد بگیریم، ترتیب مطالعه |
| [01-install-and-cargo.md](docs/01-install-and-cargo.md) | نصب (شامل راه‌های بدون اینترنت)، cargo کامل |
| [02-syntax.md](docs/02-syntax.md) | سینتکس ریزبه‌ریز: متغیر، نوع، تابع، کنترل جریان، اپراتور |
| [03-ownership.md](docs/03-ownership.md) | مالکیت، قرض، عمر، `Drop` |
| [04-types-and-traits.md](docs/04-types-and-traits.md) | انواع، struct/enum، صفت‌ها، ژنریک، تبدیل‌ها |
| [05-errors.md](docs/05-errors.md) | مدیریت خطا از صفر تا الگوهای کتابخانه‌ای |
| [06-iterators-and-closures.md](docs/06-iterators-and-closures.md) | ایتراتور، بستار، `Box<dyn Fn>` |
| [07-smart-pointers.md](docs/07-smart-pointers.md) | Box/Rc/RefCell/Weak/Arc/Cow/خودارجاعی |
| [08-concurrency.md](docs/08-concurrency.md) | ترد، کانال، قفل، اتمیک، Send/Sync، استخر ترد |
| [09-async.md](docs/09-async.md) | async/await، Future، اجراکننده، tokio — و خودمانی‌اش |
| [10-macros.md](docs/10-macros.md) | ماکرو اعلانی و رویه‌ای، اصول hygiene |
| [11-unsafe-and-ffi.md](docs/11-unsafe-and-ffi.md) | unsafe، اشاره‌گر خام، FFI، قواعد صحت |
| [12-tricks.md](docs/12-tricks.md) | ⭐ ۱۰۰ ترفند و الگوی حرفه‌ای |
| [13-pitfalls.md](docs/13-pitfalls.md) | 💣 دام‌ها و خطاهای رایج + راه‌حل |
| [14-performance.md](docs/14-performance.md) | کارایی، پروفایل، بهینه‌سازی |
| [15-std-cheatsheet.md](docs/15-std-cheatsheet.md) | مرجع سریع کتابخانهٔ استاندارد |
| [16-ecosystem.md](docs/16-ecosystem.md) | crateهای مهم و کِی از کدام استفاده کنیم |
| [17-glossary.md](docs/17-glossary.md) | واژه‌نامهٔ فارسی ↔ انگلیسی اصطلاحات |

---

## 📦 ساختار

```text
ar/
├── Cargo.toml            # شناسنامهٔ پروژه + پروفایل‌های بهینه‌سازی
├── README.md             # همین فایل (به مستندات کریت هم تزریق می‌شود!)
├── scripts/
│   ├── setup-rust.sh     # نصب Rust بدون rustup (ترفند محیط بسته)
│   └── run-web.sh        # بالا آوردن رابط وب + ترمیم خودکار مقدمات
├── src/
│   ├── main.rs           # لایهٔ نمایش: آرگومان → منطق → چاپ
│   ├── lib.rs            # ریشهٔ کتابخانه + مستندات
│   ├── model.rs          # مدل داده
│   ├── errors.rs         # خطاها
│   ├── json.rs           # JSON: نویسنده + پارسر
│   ├── store.rs          # انباره: trait + File/Memory
│   ├── app.rs            # منطق برنامه
│   ├── cli.rs            # پارسر آرگومان
│   ├── table.rs          # جدول و نوار پیشرفت
│   ├── color.rs          # رنگ ANSI
│   ├── web.rs            # سرور HTTP
│   ├── report.rs         # سازندهٔ گزارش HTML مستقل (ar html)
│   ├── web_index.html    # رابط وب (با include_str! داخل باینری)
│   └── demos/            # ۱۵ دموی آموزشی (هر موضوع = یک ماژول)
├── examples/             # نمونه‌های استفاده از کتابخانه
│   ├── quickstart.rs     # ۲۰ خط: add → done → list → stats
│   └── report.rs         # گزارش واقعی از فایل داده (با نوار و برچسب)
├── tests/
│   └── integration.rs    # ۹ تست یکپارچه روی API عمومی (فایل، JSON، خطا)
└── docs/                 # ۱۸ سند مرجع فارسی
    ├── README.md         # فهرست + مسیرهای مطالعه
    └── 00-roadmap.md … 17-glossary.md
```

---

## 🤝 مشارکت و مسیر بعدی

ایده‌هایی برای تمرین (به‌ترتیب سختی):

1. `ar edit <#> --title "…"` اضافه کن (پارسر CLI + منطق + تست).
2. قالب `--format csv` برای `list` بنویس.
3. در `web.rs` یک «رویداد زنده» با `text/event-stream` اضافه کن.
4. `src/demos/async_rt.rs`: یک اجراکنندهٔ async کوچک با `Future` دست‌ساز بنویس.
5. ذخیره‌سازی را به SQLite تغییر بده — اول ببین چقدر باید عوض شود.

## 📄 مجوز

MIT — [LICENSE](https://github.com/amirui123/ar/blob/main/LICENSE)
