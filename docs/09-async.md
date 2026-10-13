# ⚡ ۰۹ — async/await و اجراکننده‌ها

## ۱) async چیست (به‌زبان ساده)

`async fn` یک تابع است که وقتی `.await` می‌زند، **کنترل را برمی‌گرداند** و
بقیهٔ کار را بعداً ادامه می‌دهد. نتیجه‌اش یک `Future` است — یعنی «کاری که
هنوز تمام نشده».

```rust
async fn fetch_two(url1: &str, url2: &str) -> (String, String) {
    let a = fetch(url1).await;      // اینجا می‌تواند کارهای دیگر انجام شود
    let b = fetch(url2).await;
    (a, b)
}
```

⚠️ **جدا از `std::future::Future`، استاندارد Rust اجراکننده (runtime) ندارد!**
یعنی `async` به‌تنهایی هیچ کاری نمی‌کند؛ باید با `.await` در یک اجراکننده
(مثل `tokio`) یا `block_on` خودت اجرا شود. دلیل این تصمیم: بشود در
میکروکنترلر و در سرور غول استفاده شود.

---

## ۲) `Future` — قلب ماجرا

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
pub enum Poll<T> { Ready(T), Pending }
```

**قرارداد `poll`:** اگر `Pending` برگرداند، **الزاماً** باید `cx.waker()` را
نگه دارد و بعداً `wake()` کند — وگرنه اجراکننده هیچ‌وقت دوباره صدا نمی‌زند و
کار برای همیشه معلق می‌ماند (این کلاسیک‌ترین باگ نوشتن اجراکننده است).

### یک `Future` دست‌ساز (بدون هیچ crate)
```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Sleep { until: Instant }

impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.until {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();     // ← باید نگه داریم
            let until = self.until;
            std::thread::spawn(move || {        // (روش ساده؛ در واقعیت تایمر بهینه)
                let now = Instant::now();
                if until > now { std::thread::sleep(until - now); }
                waker.wake();                   // ← بیدارش کن
            });
            Poll::Pending
        }
    }
}

fn sleep(d: Duration) -> Sleep { Sleep { until: Instant::now() + d } }
```
⭐ این «۱۵ خط» کل جادوی async است: `poll` + `Waker`.

---

## ۳) اجراکنندهٔ کوچک (Block-on)

```rust
fn block_on<F: Future>(mut fut: F) -> F::Output {
    use std::sync::{Arc, Condvar, Mutex};
    use std::task::{Wake, Waker};

    struct Signal { done: Mutex<bool>, cv: Condvar }
    impl Wake for Signal {
        fn wake(self: Arc<Self>) { *self.done.lock().unwrap() = true; self.cv.notify_all(); }
        fn wake_by_ref(self: &Arc<Self>) { self.wake() }
    }

    let signal = Arc::new(Signal { done: Mutex::new(false), cv: Condvar::new() });
    let waker = Waker::from(Arc::clone(&signal));
    let mut cx = Context::from_waker(&waker);

    let mut fut = Box::pin(fut);                 // Pin برای امنیت حافظه
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                let mut done = signal.done.lock().unwrap();
                while !*done { done = signal.cv.wait(done).unwrap(); }
                *done = false;
            }
        }
    }
}
```
> 💡 `Waker::from(Arc<W>)` روی پیاده‌ساز `Wake` (stable از ۱.۵۱) جایگزین
> `RawWaker` دستی و `unsafe` شده — همین را استفاده کن.

اجرا:
```rust
let start = std::time::Instant::now();
block_on(async {
    sleep(Duration::from_millis(100)).await;
    sleep(Duration::from_millis(100)).await;
});
println!("{:?}", start.elapsed());     // ~۲۰۰ms (سری)
```

---

## ۴) هم‌زمان اجرا کردن چند Future (بدون crate)

```rust
async fn join3<A: Future, B: Future, C: Future>(a: A, b: B, c: C) -> (A::Output, B::Output, C::Output) {
    // الگوی ساده: یک ساختار که همه را نگه می‌دارد و تا آماده‌نشدن همه Pending می‌دهد
    let (mut a, mut b, mut c) = (Box::pin(a), Box::pin(b), Box::pin(c));
    let (mut ra, mut rb, mut rc) = (None, None, None);

    std::future::poll_fn(move |cx| {
        if ra.is_none() { if let Poll::Ready(v) = a.as_mut().poll(cx) { ra = Some(v); } }
        if rb.is_none() { if let Poll::Ready(v) = b.as_mut().poll(cx) { rb = Some(v); } }
        if rc.is_none() { if let Poll::Ready(v) = c.as_mut().poll(cx) { rc = Some(v); } }
        match (&ra, &rb, &rc) {
            (Some(_), Some(_), Some(_)) => Poll::Ready((ra.take().unwrap(), rb.take().unwrap(), rc.take().unwrap())),
            _ => Poll::Pending,
        }
    }).await
}
```
🎯 در دنیای واقعی این‌ها را از crate می‌گیری: `tokio::join!`, `futures::join!`,
`select!`, `FuturesUnordered`, `buffered`, `stream`.

---

## ۵) `Pin` — چرا لازم است؟

`async fn` به یک **ماشین حالت** کامپایل می‌شود که ممکن است **خودارجاع** باشد
(فیلدی به فیلد دیگرش اشاره کند). اگر آن را در حافظه جابه‌جا کنی، آن اشاره‌گر
داخلی می‌شکند → پس `Pin` جابه‌جایی را ممنوع می‌کند.

```rust
let mut fut = Box::pin(my_future());     // ✅ جابه‌جایی ممنوع
let out = fut.as_mut().poll(&mut cx);
```
* `Box::pin(x)` = روی heap پین می‌کند (راه رایج).
* `pin!` ماکرو (std) = پین روی پشته بدون تخصیص heap.
* `Unpin` = «این نوع بی‌خطر جابه‌جا می‌شود» (بیشتر انواع معمولی Unpin هستند).
* 🎯 عملاً لازم نیست خودت `Pin` را لمس کنی، مگر `poll` بنویسی.

---

## ۶) با `tokio` (دنیای واقعی)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() {
    // ۱) اجرای موازی با spawn
    let h1 = tokio::spawn(async { work(1).await });
    let h2 = tokio::spawn(async { work(2).await });
    let (a, b) = (h1.await.unwrap(), h2.await.unwrap());

    // ۲) join!/try_join!/select!
    let (x, y) = tokio::join!(fetch("a"), fetch("b"));

    // ۳) زمان
    tokio::time::sleep(Duration::from_millis(50)).await;
    let t = tokio::time::timeout(Duration::from_secs(1), fetch("c")).await;

    // ۴) IO
    let mut f = tokio::fs::File::open("x").await?;
    let mut s = String::new();
    tokio::io::AsyncReadExt::read_to_string(&mut f, &mut s).await?;

    // ۵) شبکه (همان الگوی سرور ما، ولی async)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle(socket));           // هزاران اتصال با چند ترد
    }

    // ۶) کانال بین taskها
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    // ۷) قفل async (نکته: std::sync::Mutex را بین await نگه ندار!)
    let shared = std::sync::Arc::new(tokio::sync::Mutex::new(0));
}
```

### ⚠️ دام‌های async
| دام | توضیح |
|-----|-------|
| `std::sync::Mutex` را بین `.await` نگه‌داشتن | می‌تواند بن‌بست بدهد → `tokio::sync::Mutex` |
| کار CPU-سنگین در task | کل ترد را می‌خواباند → `spawn_blocking` |
| `async` بدون `.await` | هیچ کاری انجام نمی‌شود (فقط یک Future ساخته شده!) |
| فراموش‌کردن `wake()` در executor خودت | کار برای همیشه معلق |
| لغو شدن task در وسط کار | باید «cancel-safe» بنویسی (`select!`) |
| `block_on` داخل `async fn` | بن‌بست کامل |

### لغو (cancellation)
در async، «لغو» = رهاکردن `Future`. اگر کار نیمه‌کاره انجام شده (مثلاً نصف پیام
ارسال شده)، دچار مشکل می‌شوی. راه‌حل: کارهای اتمیک، یا الگوی `select!` با شاخهٔ
پاک‌سازی، یا مستندکردن «این Future امن برای لغو است».

---

## ۷) async در برابر ترد — راهنمای انتخاب

| معیار | ترد | async |
|-------|-----|-------|
| CPU موازی | ✅ | ❌ (باید spawn_blocking) |
| ۱۰٬۰۰۰ اتصال شبکه | ❌ (پشتهٔ زیاد) | ✅ |
| کد ساده/بدون runtime | ✅ | ❌ |
| اکوسیستم بالغ (HTTP، DB) | ok | ✅✅ |
| دیباگ (backtrace خوانا) | ✅ | ⚠️ سخت‌تر |
| `select`/timeout/composition | ❌ | ✅ |

🎯 اگر تازه‌کاری: **اول ترد را یاد بگیر، بعد async.** (همین ترتیب این پوشه)

---

## ۸) تمرین پیشنهادی

`src/demos/` یک ماژول به نام `async_rt.rs` اضافه کن که:
1. `Sleep` و `block_on` بالا را داشته باشد،
2. و برای `ar serve` یک حالت `--async` بسازد که به‌جای ترد-به-ازای-اتصال،
   با یک اجراکنندهٔ کوچک کارها را روی چند ترد زمان‌بندی کند.
3. با `wrk`/`ab` یا یک اسکریپت ساده با ۵۰۰ اتصال هم‌زمان تستش کن.

این تمرین، هم `Future` را می‌فهماند، هم تفاوت واقعی ترد و async را نشان می‌دهد.

[⬅️ قبلی: هم‌روندی](08-concurrency.md) · [بعدی: ماکروها ➡️](10-macros.md)
