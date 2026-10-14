//! # دموی هم‌روندی (Concurrency) — تردها، کانال‌ها، قفل‌ها
//!
//! فلسفهٔ Rust: **«ترسِ بدون هم‌روندی»** (Fearless Concurrency).
//! دو صفت همهٔ کار را می‌کنند و کامپایلر آن‌ها را بررسی می‌کند:
//!
//! * **`Send`** → «می‌توان این مقدار را به ترد دیگری *فرستاد*»
//! * **`Sync`** → «می‌توان به این مقدار از چند ترد *اشاره* کرد»
//!
//! `Rc<T>` هیچ‌کدام نیست (شمارندهٔ غیراتمیک)، `Arc<Mutex<T>>` هر دو هست.
//! اگر اشتباه کنی، **خطای کامپایل** می‌گیری، نه باگ نادر در زمان اجرا.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::color;

/// اجرای دمو.
pub fn run() {
    println!("{}", color::bold("۱) اولین ترد — spawn و join"));
    basics();

    println!("\n{}", color::bold("۲) کانال (channel) — پیام فرستادن بین تردها"));
    channels();

    println!("\n{}", color::bold("۳) اشتراک وضعیت: Arc<Mutex<T>>"));
    shared_state();

    println!("\n{}", color::bold("۴) اتمیک‌ها — سبک‌ترین ابزار شمارش"));
    atomics();

    println!("\n{}", color::bold("۵) تردهای محدوده‌دار (scoped) — بدون Arc و بدون move"));
    scoped();

    println!("\n{}", color::bold("۶) استخر ترد — الگویی که واقعاً استفاده می‌کنی"));
    pool();
}

// ═══════════════════════════════════════════════════════════════════════════
fn basics() {
    // `spawn` یک ترد جدید می‌سازد. بستار باید `move` باشد و `'static` بماند.
    let handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("    [ترد فرزند] گام {i}");
            thread::sleep(Duration::from_millis(10));
        }
        "تمام شد" // ⬅️ مقدار برگشتی بستار = مقدار خروجی `join()`
    });

    // کار موازی در ترد اصلی
    for i in 1..=2 {
        println!("    [ترد اصلی] گام {i}");
        thread::sleep(Duration::from_millis(12));
    }

    // `join` منتظر می‌ماند و `Result` برمی‌گرداند (خطا اگر ترد پنیک کرده باشد)
    match handle.join() {
        Ok(msg) => println!("  نتیجهٔ ترد فرزند: {msg}"),
        Err(_) => println!("  ترد فرزند پنیک کرد!"),
    }

    // اطلاعات ترد
    println!("  ترد فعلی: {:?}", thread::current().name().unwrap_or("بی‌نام"));
    println!("  تعداد هسته‌های منطقی: {:?}", thread::available_parallelism().map(|n| n.get()).unwrap_or(0));

    // ترفند: `Builder` برای نام‌گذاری و اندازهٔ پشتهٔ سفارشی
    let named = thread::Builder::new()
        .name("کارگر-۱".into())
        .stack_size(512 * 1024) // ۵۱۲ کیلوبایت
        .spawn(|| "با نام")
        .unwrap();
    println!("  ترد نام‌دار: {}", named.join().unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
fn channels() {
    // کانال = یک `(Sender, Receiver)`. چند فرستنده، یک گیرنده.
    let (tx, rx) = mpsc::channel::<String>(); // mpsc = Multi-Producer, Single-Consumer

    // سه «کارگر» که پیام می‌فرستند
    let mut handles = Vec::new();
    for id in 0..3 {
        let tx = tx.clone(); // ترفند: `clone` کردن برای هر فرستنده
        handles.push(thread::spawn(move || {
            for step in 1..=2 {
                tx.send(format!("کارگر {id} گام {step}")).expect("گیرنده مرده");
                thread::sleep(Duration::from_millis(5));
            }
        }));
    }

    // 🔑 نکتهٔ حیاتی: `tx` اصلی را رها کن، وگرنه `rx` هیچ‌وقت نمی‌فهمد کار تمام شده!
    //    (`for` روی rx وقتی تمام می‌شود که *همهٔ* فرستنده‌ها رها شده باشند)
    drop(tx);

    // دریافت پیام‌ها — ترتیب بین کارگرها تضمینی نیست، ولی هر کانال FIFO است
    let mut received = Vec::new();
    while let Ok(msg) = rx.recv() {
        received.push(msg);
    }

    received.sort();
    println!("  پیام‌های دریافتی ({} مورد):", received.len());
    for m in &received {
        println!("    • {m}");
    }

    for h in handles {
        h.join().unwrap();
    }

    // ترفند: `try_recv` برای حالت غیرمسدودکننده
    let (tx2, rx2) = mpsc::channel::<i32>();
    tx2.send(7).unwrap();
    println!("  try_recv اول: {:?}", rx2.try_recv());
    println!("  try_recv دوم (خالی): {:?}", rx2.try_recv().err());

    // ترفند: `sync_channel` با ظرفیت محدود = «فشار برگشتی» (backpressure)
    let (stx, srx) = mpsc::sync_channel::<i32>(1);
    stx.send(1).unwrap(); // بلافاصله
    println!("  sync_channel ظرفیت ۱: {}", srx.recv().unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
fn shared_state() {
    // Arc = مالکیت مشترک بین تردها؛ Mutex = دسترسی انحصاری
    let counter = Arc::new(Mutex::new(0usize));
    let workers = 8;
    let increments = 1000;

    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments {
                    // 🔸 ترفند «قفل مسموم»: اگر تردی در حال نگه‌داشتن قفل پنیک کند،
                    //    قفل «مسموم» می‌شود. `unwrap_or_else(|e| e.into_inner())`
                    //    یعنی «داده را بردار و به مسمومیت اهمیت نده».
                    let mut n = counter.lock().unwrap_or_else(|e| e.into_inner());
                    *n += 1;
                    // نگهبان (guard) اینجا رها می‌شود → قفل آزاد می‌شود
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let total = *counter.lock().unwrap();
    println!("  {workers} ترد × {increments} افزایش = {total} (انتظار: {})", workers * increments);
    assert_eq!(total, workers * increments);

    // ⚡ ترفند بزرگ بهینه‌سازی: قفل را «کوتاه» بگیر.
    // ❌ بد: برای هر افزایش، lock/unlock ۱۰۰۰ بار در هر ترد
    // ✅ خوب (مثال دوم): یک بار قفل بگیر و ۱۰۰۰ کار انجام بده
    let batch = Arc::new(Mutex::new(0usize));
    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let batch = Arc::clone(&batch);
            thread::spawn(move || {
                let mut guard = batch.lock().unwrap(); // فقط یک بار
                for _ in 0..increments {
                    *guard += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("  با قفل دسته‌ای: {} (همین نتیجه، ولی قفل‌گیری کم‌تر → سریع‌تر)", *batch.lock().unwrap());

    // ⚠️ دام معروف: `if let` + قفل → نگهبان تا پایان بلوک زنده می‌ماند
    //    اگر داخلش دوباره قفل بگیری → بن‌بست (deadlock).
    println!("  ⚠️ همیشه مراقب بن‌بست باش: قفل‌ها را همیشه با یک «ترتیب ثابت» بگیر");
}

// ═══════════════════════════════════════════════════════════════════════════
fn atomics() {
    // نوع اتمیک = عملیات غیرقابل‌تقسیم در سطح سخت‌افزار. بدون قفل، بدون بن‌بست.
    static CALLS: AtomicUsize = AtomicUsize::new(0);

    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..500 {
                    CALLS.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("  AtomicUsize با ۴ ترد: {}", CALLS.load(Ordering::Relaxed));

    // 📌 راهنمای «ترتیب حافظه» (memory ordering):
    //  • Relaxed    → فقط اتمیک بودن مهم است، ترتیب با بقیهٔ حافظه نه. (شمارنده)
    //  • Acquire/Release → الگوی «قفل دستی» برای انتشار داده بین تردها.
    //  • SeqCst     → قوی‌ترین و کندترین؛ وقتی شک داری.
    println!("  (شمارنده‌ها: Relaxed کافی است — سریع‌ترین)");

    // ترفند: `compare_exchange` = «اگر مقدار فعلاً X بود، به Y تغییر بده»
    let flag = AtomicUsize::new(1);
    match flag.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => println!("  compare_exchange موفق: {old} → {}", flag.load(Ordering::Relaxed)),
        Err(cur) => println!("  ناموفق، مقدار فعلی {cur}"),
    }
    // بار دوم ناموفق می‌شود چون مقدار دیگر ۱ نیست
    println!("  بار دوم: {:?}", flag.compare_exchange(1, 3, Ordering::SeqCst, Ordering::SeqCst));

    // ترفند: `AtomicBool` به‌عنوان «پرچم توقف» (stop flag) — الگوی کلاسیک
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s2 = Arc::clone(&stop);
    let worker = thread::spawn(move || {
        let mut count = 0u64;
        while !s2.load(Ordering::Relaxed) {
            count += 1;
            thread::yield_now(); // فرصت بده به بقیه
        }
        count
    });
    thread::sleep(Duration::from_millis(5));
    stop.store(true, Ordering::Relaxed); // علامت توقف
    let spins = worker.join().unwrap();
    println!("  کارگر با پرچم توقف، {spins} دور چرخید و تمیز متوقف شد");
}

// ═══════════════════════════════════════════════════════════════════════════
fn scoped() {
    // 🎯 پایدار از Rust 1.63: تردهای محدوده‌دار.
    //    ✅ بدون `move` و بدون `Arc` → می‌توانی متغیرهای محلی را قرض بدهی.
    //    ✅ کامپایلر تضمین می‌کند همهٔ تردها قبل از پایان بلوک تمام شوند.
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let sum = thread::scope(|scope| {
        // داده را به دو نیمهٔ غیرهمپوشان تقسیم کن (borrow checker اجازه می‌دهد!)
        let (left, right) = numbers.split_at(4);

        let h1 = scope.spawn(move || left.iter().sum::<i32>());
        let h2 = scope.spawn(move || right.iter().sum::<i32>());

        h1.join().unwrap() + h2.join().unwrap()
    });

    println!("  جمع موازی بدون Arc: {sum}");
    println!("  و `numbers` هنوز سالم است: {numbers:?}");

    // نمونهٔ «اشتراک قرضی»: چند ترد فقط می‌خوانند
    let text = "متن مشترک برای شمارش کلمات".to_string();
    let counts = thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..3 {
            handles.push(scope.spawn(|| text.split_whitespace().count()));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
    });
    println!("  سه ترد، سه نتیجهٔ یکسان: {counts:?}");
}

// ═══════════════════════════════════════════════════════════════════════════
/// استخر ترد ساده — همان چیزی که سرور HTTP ما از آن استفاده می‌کند.
///
/// ایده: n ترد بساز که منتظر «کار» بنشینند روی یک کانال.
/// وقتی کار آمد، یکی از آن‌ها برش می‌دارد و اجرا می‌کند.
struct ThreadPool {
    /// فرستندهٔ کارها؛ `Option` تا در `drop` بتوانیم ببندیمش.
    sender: Option<mpsc::Sender<Job>>,
    /// نگه‌داشتن دستهٔ تردها تا زنده بمانند
    workers: Vec<thread::JoinHandle<()>>,
}

/// یک کار = بستاری که یک بار اجرا می‌شود و بین تردها فرستاده می‌شود.
///
/// `Box<dyn FnOnce() + Send + 'static>` را با دقت بخوان:
/// * `Box` → اندازه‌اش ثابت است (هر بستار اندازهٔ خودش را دارد)
/// * `dyn FnOnce()` → هر چیزی که یک بار قابل صدا زدن است
/// * `Send + 'static` → قابل فرستادن به ترد دیگر و بدون قرض محلی
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0, "اندازهٔ استخر باید بزرگ‌تر از صفر باشد");
        let (tx, rx) = mpsc::channel::<Job>();

        // 🔑 گیرنده را در `Arc<Mutex<>>` می‌گذاریم تا چند کارگر بتوانند از آن بکشند
        let rx = Arc::new(Mutex::new(rx));

        let workers = (0..size)
            .map(|id| {
                let rx = Arc::clone(&rx);
                thread::spawn(move || {
                    loop {
                        // قفل فقط برای گرفتنِ کار؛ بعد آزادش می‌کنیم
                        let job = {
                            let guard = rx.lock().unwrap_or_else(|e| e.into_inner());
                            guard.recv() // Err یعنی همهٔ فرستنده‌ها رفتند → ترد تعطیل می‌شود
                        };

                        match job {
                            Ok(job) => {
                                println!("    [کارگر {id}] کار را برداشت");
                                job(); // اجرا
                            }
                            Err(_) => break, // کانال بسته شد
                        }
                    }
                })
            })
            .collect();

        ThreadPool { sender: Some(tx), workers }
    }

    /// سپردن کار به استخر.
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(tx) = &self.sender {
            tx.send(Box::new(f)).expect("کارگرها رفته‌اند");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // فرستنده را می‌بندیم → کارگرها `Err` می‌گیرند و حلقه را تمام می‌کنند
        self.sender = None;
        // سپس منتظر می‌مانیم همه تمام شوند
        for w in self.workers.drain(..) {
            let _ = w.join();
        }
        println!("    (استخر تمیز تعطیل شد)");
    }
}

fn pool() {
    let pool = ThreadPool::new(3);

    for i in 1..=6 {
        pool.execute(move || {
            thread::sleep(Duration::from_millis(5));
            println!("      کار {i} تمام شد");
        });
    }

    // یک کار با وضعیت مشترک
    let total = Arc::new(AtomicUsize::new(0));
    for _ in 0..4 {
        let total = Arc::clone(&total);
        pool.execute(move || {
            total.fetch_add(10, Ordering::Relaxed);
        });
    }

    drop(pool); // تعطیلی تمیز (Drop بالا)
    println!("  حاصل کارهای مشترک: {}", total.load(Ordering::Relaxed));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        run();
    }

    #[test]
    fn mutex_counter_is_exact() {
        let c = Arc::new(Mutex::new(0));
        let hs: Vec<_> = (0..4)
            .map(|_| {
                let c = Arc::clone(&c);
                thread::spawn(move || {
                    for _ in 0..250 {
                        *c.lock().unwrap() += 1;
                    }
                })
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(*c.lock().unwrap(), 1000);
    }

    #[test]
    fn channel_receives_all_messages() {
        let (tx, rx) = mpsc::channel();
        let hs: Vec<_> = (0..3)
            .map(|i| {
                let tx = tx.clone();
                thread::spawn(move || tx.send(i).unwrap())
            })
            .collect();
        drop(tx);
        let sum: i32 = rx.iter().sum();
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(sum, 0 + 1 + 2);
    }

    #[test]
    fn thread_pool_runs_all_jobs() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..10 {
            let c = Arc::clone(&counter);
            pool.execute(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }
        drop(pool); // join تضمین‌شده
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn scoped_threads_sum_correctly() {
        let v = vec![1, 2, 3, 4];
        let s = thread::scope(|sc| {
            let (l, r) = v.split_at(2);
            let a = sc.spawn(move || l.iter().sum::<i32>());
            let b = sc.spawn(move || r.iter().sum::<i32>());
            a.join().unwrap() + b.join().unwrap()
        });
        assert_eq!(s, 10);
    }

    #[test]
    fn atomic_compare_exchange_semantics() {
        let a = AtomicUsize::new(1);
        assert!(a.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst).is_ok());
        assert!(a.compare_exchange(1, 3, Ordering::SeqCst, Ordering::SeqCst).is_err());
        assert_eq!(a.load(Ordering::SeqCst), 2);
    }
}
