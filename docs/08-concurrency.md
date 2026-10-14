# 🧵 ۰۸ — هم‌روندی (Concurrency)

## ۱) مدل ذهنی

Rust دو صفت خودکار دارد که همهٔ داستان را می‌سازند:

| صفت | معنی | مثال |
|-----|------|------|
| `Send` | می‌شود **فرستاد** به ترد دیگر | `String`, `Vec<T>`, `Arc<T: Send+Sync>` |
| `Sync` | می‌شود **از چند ترد به آن اشاره کرد** (`&T`) | `Mutex<T: Send>`, `AtomicUsize` |
| ❌ `Rc<T>` | نه Send، نه Sync | شمارندهٔ غیراتمیک |
| ❌ `RefCell<T>` | Sync نیست | قرض‌گیری غیراتمیک |
| ✅ `Arc<Mutex<T>>` | هم Send، هم Sync | جایگزین امن |

خطای «`Rc<T>` cannot be sent between threads safely» یعنی **کامپایلر جلوی یک
باگ زمان اجرا را گرفت** — به آن افتخار کن، نه شکایت.

---

## ۲) تردهای ساده

```rust
let handle = std::thread::spawn(move || {     // ← move لازم است
    let sum: i32 = data.iter().sum();
    sum * 2
});
let result = handle.join().expect("ترد پنیک کرد");   // ← Result می‌دهد
```

* `spawn` نیاز به `F: FnOnce() -> T + Send + 'static` دارد:
  * `Send` = بستار و محیطش قابل انتقال،
  * `'static` = هیچ مرجع کوتاه‌عمری داخلش نیست.
* `Builder::new().name(..).stack_size(..).spawn(..)` برای تنظیمات.
* `thread::available_parallelism()` = تعداد هسته‌های منطقی.
* `thread::sleep(Duration)` / `thread::yield_now()`.

### 🎯 `std::thread::scope` — ترفند مدرن (Rust 1.63+)
```rust
let data = vec![1, 2, 3, 4];
let total = std::thread::scope(|s| {
    let (left, right) = data.split_at(2);      // ← دو برش غیرهمپوشان
    let a = s.spawn(move || left.iter().sum::<i32>());   // هزینهٔ +move
    let b = s.spawn(move || right.iter().sum::<i32>());
    a.join().unwrap() + b.join().unwrap()
});
```
✅ نیازی به `Arc` نیست — تردها دادهٔ محلی را **قرض** می‌گیرند.
✅ کامپایلر تضمین می‌کند همه قبل از پایان `scope` تمام شوند.

---

## ۳) کانال‌ها (message passing)

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel::<String>();      // Multi-Producer, Single-Consumer

for id in 0..3 {
    let tx = tx.clone();
    thread::spawn(move || { tx.send(format!("کارگر {id}")).unwrap(); });
}
drop(tx);                                       // 🔑 وگرنه rx هرگز تمام نمی‌شود!

for msg in rx { println!("{msg}"); }            // تا بسته‌شدن همهٔ فرستنده‌ها
// یا: while let Ok(msg) = rx.recv() { }
// یا: rx.try_recv()  → غیرمسدودکننده
// یا: rx.recv_timeout(Duration::from_millis(100))
```

| نوع | ویژگی |
|-----|-------|
| `channel()` | نامحدود — تولیدکننده هرگز بلاک نمی‌شود (خطر مصرف حافظه) |
| `sync_channel(n)` | محدود — **فشار برگشتی (backpressure)** ✅ |
| `sync_channel(0)` | هم‌زمان‌سازی کامل (rendezvous) |

🎯 کانال برای «مالکیت را منتقل کن، داده را نچرخان» عالی است؛ اغلب بهتر از
قفل، چون از بن‌بست و شرایط رقابتی پیشگیری می‌کند. (شعار: *Don't communicate by
sharing memory; share memory by communicating.*)

---

## ۴) وضعیت مشترک

```rust
let state = Arc::new(Mutex::new(Vec::<String>::new()));
let s = Arc::clone(&state);
thread::spawn(move || {
    let mut g = s.lock().unwrap_or_else(|e| e.into_inner());
    g.push("سلام".into());
});   // ← نگهبان اینجا آزاد می‌شود
```

| ابزار | کِی |
|-------|-----|
| `Mutex<T>` | نوشتن/خواندن — همهٔ حالت‌ها |
| `RwLock<T>` | خواندن‌های موازی زیاد، نوشتن کم |
| `AtomicUsize/Bool/…` | شمارنده و فلگ (بدون قفل، سریع‌ترین) |
| `mpsc` | انتقال داده بین تردها |
| `OnceLock`/`LazyLock` | مقداردهی یک‌بارهٔ سراسری |

⚠️ **قواعد ضدبن‌بست:** قفل را کوتاه نگه دار · ترتیب ثابت · هرگز داخل قفل
توسط کدی که ممکن است همان قفل را بگیرد فراخوانی نکن · `try_lock` برای حالت‌های
حساس.

---

## ۵) استخر ترد — الگویی که واقعاً لازم می‌شود

```rust
type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));             // ← چند کارگر، یک گیرنده
        let workers = (0..size).map(|_| {
            let rx = Arc::clone(&rx);
            thread::spawn(move || loop {
                let job = { rx.lock().unwrap().recv() };   // ← قفل فقط برای برداشتن
                match job { Ok(job) => job(), Err(_) => break }
            })
        }).collect();
        ThreadPool { sender: Some(tx), workers }
    }

    fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.as_ref().unwrap().send(Box::new(f)).expect("کارگرها رفته‌اند");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender = None;                            // ← بستن کانال → کارگرها تمام می‌شوند
        for w in self.workers.drain(..) { let _ = w.join(); }
    }
}
```
📌 نکات کلیدی این الگو:
* `Box<dyn FnOnce() + Send + 'static>`: اندازهٔ ثابت + هر بستاری + قابل انتقال.
* `sender = None` در `Drop` = «پایان کار، برو خانه».
* «قفل برای برداشتن کار، آزاد برای اجرا» → کارگرها موازی کار می‌کنند.
* سرور `web.rs` ما از همین ساختار الگو گرفته (به‌صورت ترد-به-ازای-اتصال برای سادگی).

---

## ۶) async چطور در این تصویر جا می‌شود؟

| | ترد (thread) | async |
|---|---|---|
| مناسب برای | کار CPU-محور، تعداد کم | IO-محور، تعداد زیاد (۱۰k+ اتصال) |
| هزینهٔ هر واحد | مگابایت پشته، سوئیچ OS | چند صد بایت، سوئیچ در کاربر |
| پیش‌فعالی | OS | باید خودت اجراکننده داشته باشی (`tokio`/`async-std`) |
| اشتراک داده | `Arc<Mutex<T>>` | `&mut` بدون قفل (بین `.await`ها تک‌تردی) |

🎯 قانون عملی: **IO زیاد و سبک → async**، **محاسبهٔ سنگین → ترد**
(یا `tokio::task::spawn_blocking`).

---

## ۷) راهنمای سریع انتخاب

| مسئله | راه‌حل |
|-------|--------|
| کار موازی روی برش داده | `thread::scope` |
| جمع‌کردن نتایج از N کارگر | `mpsc` |
| شمارندهٔ مشترک | `Arc<AtomicUsize>` |
| نقشهٔ مشترک قابل تغییر | `Arc<Mutex<HashMap<…>>>` یا `Arc<RwLock<…>>` |
| استخر کارگر با صف کار | ThreadPool + کانال |
| خواندن‌های زیاد، نوشتن کم | `Arc<RwLock<T>>` |
| ده‌هزار اتصال شبکه | `tokio` + `async` |
| کار CPU-سنگین روی async | `spawn_blocking` / `rayon` |

---

## ۸) ابزارهای تشخیص

```bash
cargo +nightly miri test            # تشخیص داده‌های رقابتی و UB
cargo test -- --test-threads=1      # تست‌های تک‌نخی (برای دیباگ)
RUSTFLAGS="-Z sanitizer=thread" RUSTC_BOOTSTRAP=1 cargo +nightly test   # TSan
cargo build --release && perf stat ./target/release/ar demo all         # پروفایل
```
🎯 `loom` برای تست مدل‌های هم‌روندی پیچیده (crate جدا).

---

## ۹) دام‌های معروف

| دام | چرا |
|-----|-----|
| فراموش‌کردن `drop(tx)` | `rx` هیچ‌وقت تمام نمی‌شود → برنامه هنگ |
| نگه‌داشتن قفل در `if let` | قفل تا پایان بلوک می‌ماند |
| `join` نکردن تردها | کار نیمه‌تمام، خطاهای گم‌شده |
| `Arc<Mutex<Vec>>` برای هر گروهِ کوچک | رقابت قفل → کندتر از تک‌تردی (دسته‌بندی کن) |
| `Rc` با `thread::spawn` | کامپایل نمی‌شود (خوب است!) |
| `Ordering::Relaxed` برای انتشار داده | خواندن مقدار قدیمی |
| شکستن یک قفل بزرگ به دو تا بدون ترتیب | بن‌بست |
| خوابیدن (`sleep`) داخل قفل | همهٔ تردها منتظر |

[⬅️ قبلی: اشاره‌گرهای هوشمند](07-smart-pointers.md) · [بعدی: async ➡️](09-async.md)
