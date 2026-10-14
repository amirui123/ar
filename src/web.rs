//! # 🌐 سرور HTTP — از صفر، فقط با `std::net`
//!
//! این ماژول یک سرور HTTP/1.1 واقعی است که:
//! * روی `TcpListener` گوش می‌دهد،
//! * درخواست را **دستی** پارس می‌کند (خط درخواست + سرایندها + بدنه)،
//! * بر اساس مسیر (route) تصمیم می‌گیرد،
//! * و پاسخ را می‌نویسد.
//!
//! ## چیزهایی که اینجا یاد می‌گیری
//! * کار با `TcpListener` / `TcpStream` و خواندن بایت‌به‌بایت
//! * `BufReader` و چرا خواندن خط‌به‌خط بدون آن فاجعه است
//! * اشتراک وضعیت بین تردها با `Arc<Mutex<T>>`
//! * نوشتن پاسخ با `Content-Length` و `charset=utf-8` (برای فارسی حیاتی است!)
//! * الگوی «تقسیم مسیر» (routing) و پارس query string
//!
//! ## چرا خودمان نوشتیم؟
//! در دنیای واقعی از `axum` یا `actix-web` استفاده کن — ولی *یک بار* نوشتن
//! سرور خام، تازه آدم را می‌فهماند که آن فریم‌ورک‌ها چه کاری برایت می‌کنند.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::{Filter, SortKey, StatusFilter};
use crate::cli::Args;
use crate::errors::{AppError, Result};
use crate::json::{self, Json};
use crate::model::{Priority, TaskId};
use crate::store::task_to_json;
use crate::{App, color};

/// صفحهٔ HTML که در باینری «حک» می‌شود (مثل فایل جدا در زمان اجرا لازم نیست).
const INDEX_HTML: &str = include_str!("web_index.html");

/// حداکثر اندازهٔ بدنهٔ درخواست (۲ مگابایت) — محافظت ساده در برابر سوءاستفاده.
const MAX_BODY: usize = 2 * 1024 * 1024;

/// اجرای سرور تا وقتی کاربر Ctrl+C بزند.
///
/// # خطاها
/// اگر پورت قابل bind نباشد (اشغال یا دسترسی).
pub fn serve(app: App, args: &Args) -> Result<ExitCode> {
    let port: u16 = args.number::<u16>(&["port", "p"])?.unwrap_or(8080);
    // ⚠️ bind روی 0.0.0.0 (نه 127.0.0.1) تا از بیرون هم قابل دسترسی باشد.
    //    داخل کانتینر/سندباکس، این تفاوت «کار می‌کند» و «کار نمی‌کند» است!
    let host = args.flag(&["host"]).unwrap_or("0.0.0.0");
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| AppError::BadInput(format!("آدرس «{host}:{port}» نامعتبر است: {e}")))?;

    let listener = TcpListener::bind(addr)
        .map_err(|e| AppError::io(format!("گوش دادن روی {addr}"), e))?;

    // وضعیت مشترک: Arc (شمارش‌گر مشترک بین تردها) + Mutex (دسترسی انحصاری)
    let state = Arc::new(Mutex::new(app));

    println!("{} سرور بالا آمد: {}", color::green("✔"), color::bold(&format!("http://{addr}")));
    println!("  انباره: {}", state.lock().unwrap_or_else(|e| e.into_inner()).store_name());
    println!("  {}", color::dim("برای خروج: Ctrl+C"));
    println!("  {}", color::dim("نکته: صفحهٔ وب با fetch به همین سرور وصل می‌شود (بدون CDN و بدون اینترنت)"));

    // حلقهٔ پذیرش: برای هر اتصال یک ترد (برای آموزش سادگی مهم‌تر از بهینه‌سازی است)
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    // اگر خطایی رخ دهد فقط چاپ می‌کنیم؛ سرور نمی‌خوابد
                    if let Err(e) = handle_connection(stream, state) {
                        eprintln!("{} {e}", color::dim("[اتصال]"));
                    }
                });
            }
            Err(e) => {
                // خطای پذیرش معمولاً موقتی است (مثلاً قطع ناگهانی کلاینت)
                eprintln!("{} پذیرش اتصال شکست خورد: {e}", color::red("⚠"));
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

// ═══════════════════════════════════════════════════════════════════════════
//  ساختارهای درخواست/پاسخ
// ═══════════════════════════════════════════════════════════════════════════

/// یک درخواست HTTP ساده‌شده.
#[derive(Debug)]
struct Request {
    /// متد: GET, POST, DELETE, OPTIONS
    method: String,
    /// مسیر بدون query: مثل `/api/tasks/3`
    path: String,
    /// رشتهٔ پرس‌وجو: مثل `status=pending&limit=5`
    query: String,
    /// بدنه به‌صورت بایت (برای پارس JSON)
    body: Vec<u8>,
}

impl Request {
    /// گرفتن یک پارامتر از query string (بدون وابستگی به هیچ crate).
    ///
    /// ترفند: `split('&')` + `split_once('=')` یک پارسر query کامل می‌سازد.
    /// (در دنیای واقعی باید percent-decoding هم انجام دهی.)
    fn param(&self, key: &str) -> Option<String> {
        self.query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == key).then(|| urldecode(v))
        })
    }

    /// پارس بدنه به‌عنوان JSON (اگر بود).
    fn json_body(&self) -> Result<Json> {
        let text = std::str::from_utf8(&self.body)
            .map_err(|e| AppError::BadInput(format!("بدنهٔ درخواست UTF-8 معتبر نیست: {e}")))?;
        if text.trim().is_empty() {
            return Ok(Json::Obj(Vec::new()));
        }
        Ok(json::parse(text)?)
    }

    /// اجزای مسیر را جدا می‌کند: `/api/tasks/3/toggle` → `["api","tasks","3","toggle"]`
    fn segments(&self) -> Vec<&str> {
        self.path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect()
    }
}

/// یک پاسخ HTTP.
struct Response {
    /// کد وضعیت (200، 404، …)
    status: u16,
    /// نوع محتوا (به‌همراه charset برای متن‌های فارسی!)
    content_type: &'static str,
    /// بدنهٔ پاسخ
    body: Vec<u8>,
}

impl Response {
    /// پاسخ JSON.
    fn json(value: &Json, status: u16) -> Self {
        Response { status, content_type: "application/json; charset=utf-8", body: value.to_string().into_bytes() }
    }

    /// پاسخ HTML.
    fn html(text: &str) -> Self {
        Response { status: 200, content_type: "text/html; charset=utf-8", body: text.as_bytes().to_vec() }
    }

    /// پاسخ متن ساده.
    fn text(text: &str, status: u16) -> Self {
        Response { status, content_type: "text/plain; charset=utf-8", body: text.as_bytes().to_vec() }
    }

    /// پاسخ خطا به‌صورت JSON (تا فرانت‌اند راحت بخواند).
    fn error(status: u16, message: &str) -> Self {
        Response::json(&Json::obj([("error", Json::from(message))]), status)
    }

    /// ۴۰۴
    fn not_found() -> Self {
        Response::error(404, "این مسیر وجود ندارد")
    }

    /// علت کد وضعیت را به متن تبدیل می‌کند.
    fn reason(&self) -> &'static str {
        match self.status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            413 => "Payload Too Large",
            500 => "Internal Server Error",
            _ => "OK",
        }
    }

    /// تبدیل به بایت‌های کامل پاسخ HTTP.
    ///
    /// ⚠️ نکتهٔ کلیدی: `Content-Length` باید **تعداد بایت** باشد، نه کاراکتر.
    /// برای متن فارسی («سلام» = ۸ بایت ولی ۴ کاراکتر) این تفاوت حیاتی است —
    /// اگر اشتباه بدهی، مرورگر بدنه را ناقص می‌خواند.
    ///
    /// 🔑 `omit_body` برای پاسخ به درخواست `HEAD` است: طبق RFC 9110، پاسخ HEAD
    /// باید **همان سرایندهای GET** را داشته باشد (شامل `Content-Length` واقعی)
    /// ولی **بدنه‌ای نداشته باشد**. اگر برای HEAD اصلاً پاسخ ندهیم (۴۰۴)،
    /// ابزارهای سلامتی‌سنجی و پروکسی‌ها فکر می‌کنند سرویسی اینجا نیست.
    fn to_bytes(&self, omit_body: bool) -> Vec<u8> {
        let head = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             Cache-Control: no-store\r\n\
             Connection: close\r\n\
             \r\n",
            self.status,
            self.reason(),
            self.content_type,
            self.body.len()
        );

        let body = if omit_body { &[][..] } else { &self.body[..] };
        let mut out = Vec::with_capacity(head.len() + body.len());
        out.extend_from_slice(head.as_bytes());
        out.extend_from_slice(body);
        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  پارس درخواست و مدیریت اتصال
// ═══════════════════════════════════════════════════════════════════════════

/// خواندن و پردازش یک اتصال تا آخر.
fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<App>>) -> Result<()> {
    // 🌟 ترفند مهم: `&TcpStream` هم `Read` است هم `Write`.
    //    پس می‌توانیم هم‌زمان یک خوانندهٔ بافرشده و یک نویسنده داشته باشیم
    //    بدون آنکه مالکیت سوکت را جابه‌جا کنیم.
    let request = {
        let mut reader = BufReader::new(&stream);
        match read_request(&mut reader) {
            Ok(Some(req)) => req,
            Ok(None) => return Ok(()), // کلاینت فقط اتصال را باز و بست (مثل probe)
            Err(e) => {
                // پاسخ ۴۰۰ بده و تمام
                let resp = Response::error(400, &format!("درخواست نامعتبر: {e}"));
                stream.write_all(&resp.to_bytes(false)).ok();
                return Ok(());
            }
        }
    };

    // OPTIONS (پیش‌پرواز CORS) → پاسخ خالی
    if request.method == "OPTIONS" {
        let resp = Response { status: 204, content_type: "text/plain; charset=utf-8", body: Vec::new() };
        stream.write_all(&resp.to_bytes(false)).ok();
        return Ok(());
    }

    // 🔑 ترفند HEAD: به‌جای نوشتن یک شاخهٔ جداگانه برای هر مسیر، متد را به
    //    GET «نرمال» می‌کنیم و فقط یادمان می‌ماند که بدنه را ننویسیم.
    //    این‌طور `HEAD /`، `HEAD /api/tasks`، `HEAD /health` و … خودبه‌خود
    //    همان کد وضعیت و سرایندهای GET را می‌دهند (رفتار درست HTTP).
    let mut request = request;
    let head_only = request.method == "HEAD";
    if head_only {
        request.method = "GET".to_string();
    }

    println!("  {} {} {}", color::dim("→"), color::cyan(&request.method), request.path);

    let response = route(&request, &state);
    stream.write_all(&response.to_bytes(head_only))?;
    stream.flush()?;
    Ok(())
}

/// خواندن یک درخواست HTTP.
fn read_request(reader: &mut BufReader<&TcpStream>) -> std::result::Result<Option<Request>, String> {
    let mut line = String::new();

    // ── خط اول: `GET /api/tasks?x=1 HTTP/1.1` ──
    let n = reader.read_line(&mut line).map_err(|e| format!("خطا در خواندن: {e}"))?;
    if n == 0 {
        return Ok(None); // اتصال خالی
    }
    let mut parts = line.trim_end().split(' ');
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();
    if method.is_empty() {
        return Err("خط درخواست نامعتبر".into());
    }

    // جدا کردن مسیر از query
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };

    // ── سرایندها ──
    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header).map_err(|e| format!("خطا در سرایندها: {e}"))?;
        if n == 0 || header == "\r\n" || header == "\n" {
            break; // خط خالی = پایان سرایندها
        }
        // جدا کردن نام و مقدار (بی‌توجه به بزرگی/کوچکی حروف)
        if let Some((name, value)) = header.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().map_err(|_| "Content-Length نامعتبر".to_string())?;
            }
        }
    }

    if content_length > MAX_BODY {
        return Err("بدنهٔ درخواست خیلی بزرگ است".into());
    }

    // ── بدنه (دقیقاً content_length بایت) ──
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).map_err(|e| format!("خطا در خواندن بدنه: {e}"))?;
    }

    Ok(Some(Request { method, path, query, body }))
}

// ═══════════════════════════════════════════════════════════════════════════
//  مسیریابی (routing)
// ═══════════════════════════════════════════════════════════════════════════

/// تصمیم‌گیری بر اساس مسیر و متد.
fn route(req: &Request, state: &Arc<Mutex<App>>) -> Response {
    let segs = req.segments();

    match (req.method.as_str(), segs.as_slice()) {
        // صفحهٔ وب
        // 🔑 `/` و `/index.html` هر دو صفحهٔ اصلی را می‌دهند: بعضی پروکسی‌ها و
        //    مرورگرها مستقیم سراغ `/index.html` می‌روند و اگر ۴۰۴ بگیرند،
        //    پیش‌نمایش خالی می‌ماند.
        ("GET", []) | ("GET", ["index.html"]) => Response::html(INDEX_HTML),
        ("GET", ["favicon.ico"]) => Response::text("", 204),
        ("GET", ["health"]) => Response::json(&Json::obj([("ok", Json::from(true))]), 200),

        // ── API: فهرست تسک‌ها ──
        ("GET", ["api", "tasks"]) => {
            let app = lock(state);
            let filter = Filter {
                status: req.param("status").and_then(|s| StatusFilter::parse(&s).ok()).unwrap_or_default(),
                tag: req.param("tag"),
                search: req.param("q"),
                sort: req.param("sort").and_then(|s| SortKey::parse(&s).ok()).unwrap_or_default(),
                reverse: req.param("reverse").is_some_and(|v| v == "1" || v == "true"),
                limit: req.param("limit").and_then(|v| v.parse().ok()),
            };
            let list = app.list(&filter);
            let arr = Json::Arr(list.iter().map(|t| task_to_json(t)).collect());
            // پاسخ «پوشیده» با متادیتا (سبک خوب API)
            Response::json(
                &Json::obj([
                    ("count", Json::from(list.len())),
                    ("tasks", arr),
                ]),
                200,
            )
        }

        // ── API: ساخت تسک ──
        ("POST", ["api", "tasks"]) => {
            let body = match req.json_body() {
                Ok(b) => b,
                Err(e) => return Response::error(400, &e.to_string()),
            };

            let title = body.get_str_or("title", "").trim().to_string();
            if title.is_empty() {
                return Response::error(400, "فیلد «title» لازم است");
            }
            let priority = body
                .get("priority")
                .and_then(Json::as_str)
                .and_then(|s| s.parse::<Priority>().ok())
                .unwrap_or_default();

            // برچسب‌ها: هم رشتهٔ ویرگولی، هم آرایهٔ JSON
            let tags: Vec<String> = match body.get("tags") {
                Some(Json::Arr(a)) => a.iter().filter_map(|t| t.as_str().map(str::to_string)).collect(),
                Some(Json::Str(s)) => s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect(),
                _ => Vec::new(),
            };

            let mut app = lock(state);
            match app.add(&title, priority, tags) {
                Ok(id) => {
                    let _ = app.save(); // اگر خطا داد، تسک در حافظه می‌ماند (صادق‌تر از نادیده‌گرفتن)
                    match app.get(id) {
                        Ok(task) => Response::json(&task_to_json(task), 201),
                        Err(e) => Response::error(500, &e.to_string()),
                    }
                }
                Err(e) => Response::error(400, &e.to_string()),
            }
        }

        // ── API: اقدام روی یک تسک (/api/tasks/:id/…) ──
        ("POST" | "DELETE", ["api", "tasks", id, rest @ ..]) => {
            let id = match id.parse::<TaskId>() {
                Ok(v) => v,
                Err(e) => return Response::error(400, &e.to_string()),
            };

            let action = rest.first().copied().unwrap_or("");
            let mut app = lock(state);
            let result = match (req.method.as_str(), action) {
                ("DELETE", _) => app.remove(id).map(|t| task_to_json(&t)),
                ("POST", "toggle") => {
                    let done = app.get(id).map(|t| !t.done);
                    match done {
                        Ok(d) => app.set_done(id, d).and_then(|_| app.get(id)).map(task_to_json),
                        Err(e) => Err(e),
                    }
                }
                ("POST", "done") => app.set_done(id, true).and_then(|_| app.get(id)).map(task_to_json),
                ("POST", "undo") => app.set_done(id, false).and_then(|_| app.get(id)).map(task_to_json),
                _ => return Response::error(405, "این متد/اقدام مجاز نیست"),
            };

            match result {
                Ok(v) => {
                    let _ = app.save();
                    Response::json(&v, 200)
                }
                Err(AppError::TaskNotFound(_)) => Response::error(404, "تسکی با این شناسه وجود ندارد"),
                Err(e) => Response::error(400, &e.to_string()),
            }
        }

        // ── API: آمار ──
        ("GET", ["api", "stats"]) => {
            let app = lock(state);
            let s = app.stats();
            let by_priority = Json::Obj(
                s.by_priority.iter().map(|(p, n)| (p.as_str().to_string(), Json::from(*n))).collect(),
            );
            let mut tags: Vec<(&String, &usize)> = s.by_tag.iter().collect();
            tags.sort_by(|a, b| b.1.cmp(a.1));
            Response::json(
                &Json::obj([
                    ("total", Json::from(s.total)),
                    ("done", Json::from(s.done)),
                    ("pending", Json::from(s.pending)),
                    ("progress_percent", Json::from(s.progress_percent)),
                    ("by_priority", by_priority),
                    ("by_tag", Json::Obj(tags.into_iter().map(|(k, v)| (k.clone(), Json::from(*v))).collect())),
                ]),
                200,
            )
        }

        // ── API: پاک‌سازی انجام‌شده‌ها ──
        ("POST", ["api", "purge"]) => {
            let mut app = lock(state);
            let n = app.purge_done();
            let _ = app.save();
            Response::json(&Json::obj([("removed", Json::from(n))]), 200)
        }

        // ── تفکیک «متد غلط» از «مسیر غلط» ──
        // اگر متد یکی از متدهای شناخته‌شده نیست → ۴۰۵ (شرمنده، این متد را
        // پشتیبانی نمی‌کنیم). در غیر این صورت مسیر واقعاً وجود ندارد → ۴۰۴.
        (m, _) if !matches!(m, "GET" | "POST" | "DELETE") => {
            Response::error(405, "این متد پشتیبانی نمی‌شود")
        }
        _ => Response::not_found(),
    }
}

/// قفل گرفتن با گذشت از «مسمومیت» (اگر تردی هنگام نگه‌داشتن قفل پنیک کرد).
fn lock(state: &Arc<Mutex<App>>) -> std::sync::MutexGuard<'_, App> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}

/// رمزگشایی درصدی (percent-decoding) برای مقادیر query.
///
/// `%20` → فاصله، `%D8%B3` → «س» و …
fn urldecode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' '); // در query string، «+» یعنی فاصله
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    // اگر بایت‌ها UTF-8 معتبر بودند رشته می‌شود؛ وگرنه با lossy
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// یک وضعیت تازه برای تست‌ها (انبارهٔ حافظه‌ای، بدون دست‌زدن به فایل).
    fn test_state() -> Arc<Mutex<App>> {
        Arc::new(Mutex::new(App::open(crate::store::open_store(None)).unwrap()))
    }

    fn req(method: &str, target: &str) -> Request {
        let (path, query) = match target.split_once('?') {
            Some((p, q)) => (p.to_string(), q.to_string()),
            None => (target.to_string(), String::new()),
        };
        Request { method: method.to_string(), path, query, body: Vec::new() }
    }

    #[test]
    fn routing_and_params() {
        let r = req("GET", "/api/tasks?status=done&limit=3&q=%D8%B3%D9%84%D8%A7%D9%85");
        assert_eq!(r.segments(), vec!["api", "tasks"]);
        assert_eq!(r.param("status").as_deref(), Some("done"));
        assert_eq!(r.param("limit").as_deref(), Some("3"));
        assert_eq!(r.param("q").as_deref(), Some("سلام")); // percent-decoding
        assert_eq!(r.param("nope"), None);
    }

    #[test]
    fn urldecode_handles_plus_and_bad_escapes() {
        assert_eq!(urldecode("a+b"), "a b");
        assert_eq!(urldecode("%41%42"), "AB");
        assert_eq!(urldecode("%ZZ"), "%ZZ"); // ورودی خراب → همان‌طور برگردانده می‌شود
        assert_eq!(urldecode("بدون%20تغییر"), "بدون تغییر");
    }

    #[test]
    fn response_content_length_is_bytes_not_chars() {
        let resp = Response::text("سلام", 200);
        let bytes = resp.to_bytes(false);
        let head = String::from_utf8_lossy(&bytes[..bytes.len() - resp.body.len()]).to_string();
        // طول «سلام» = ۸ بایت، نه ۴ کاراکتر
        assert!(head.contains("Content-Length: 8"), "{head}");
    }

    #[test]
    fn json_body_parsing() {
        let mut r = req("POST", "/api/tasks");
        r.body = br#"{"title":"x","priority":"high"}"#.to_vec();
        let v = r.json_body().unwrap();
        assert_eq!(v.get_str_or("title", ""), "x");
    }

    #[test]
    fn empty_body_is_empty_object() {
        let r = req("POST", "/api/tasks");
        assert!(r.json_body().unwrap().as_object().unwrap().is_empty());
    }

    /// 🔑 HEAD باید مثل GET جواب بدهد ولی بدنه نداشته باشد.
    ///
    /// چرا تست مهمی است؟ چون پروکسی‌های پیش‌نمایش/سلامتی‌سنجی زیاد HEAD
    /// می‌زنند؛ اگر ۴۰۴ بگیرند، سرویس را نمی‌بینند و صفحه خالی می‌ماند.
    #[test]
    fn head_like_get_but_without_body() {
        let state = test_state();

        // پاسخ GET و پاسخ «GETِ بدون بدنه» (چیزی که برای HEAD می‌فرستیم)
        let get = route(&req("GET", "/"), &state);
        let head = get.to_bytes(true);

        let head_text = String::from_utf8_lossy(&head);
        let (headers, body) = head_text.split_once("\r\n\r\n").expect("سرایند و بدنه");
        assert!(body.is_empty(), "پاسخ HEAD نباید بدنه داشته باشد");
        assert!(
            headers.contains(&format!("Content-Length: {}", get.body.len())),
            "طول اعلام‌شده در HEAD باید همان طول واقعی GET باشد:\n{headers}"
        );
        assert!(head_text.starts_with("HTTP/1.1 200 OK"));

        // و خودِ GET باید بدنه داشته باشد (وگرنه صفحه‌ای نیست)
        assert!(!get.body.is_empty());
        assert!(String::from_utf8_lossy(&get.body).contains("<html"));
    }

    #[test]
    fn routes_return_expected_status() {
        let state = test_state();

        // صفحهٔ اصلی — از هر دو مسیر
        assert_eq!(route(&req("GET", "/"), &state).status, 200);
        assert_eq!(route(&req("GET", "/index.html"), &state).status, 200);
        // مسیر ناشناخته
        assert_eq!(route(&req("GET", "/nope"), &state).status, 404);
        // متد پشتیبانی‌نشده → ۴۰۵ (نه ۴۰۴؛ API درست همین را می‌دهد)
        assert_eq!(route(&req("PUT", "/api/tasks/1"), &state).status, 405);
        assert_eq!(route(&req("PATCH", "/"), &state).status, 405);
        // ساخت تسک بدون عنوان → ۴۰۰
        assert_eq!(route(&req("POST", "/api/tasks"), &state).status, 400);
        // فهرست
        assert_eq!(route(&req("GET", "/api/tasks"), &state).status, 200);

        // چرخهٔ کامل: ساخت → بازخوانی → تغییر وضعیت → حذف
        let mut create = req("POST", "/api/tasks");
        create.body = r#"{"title":"از تست","priority":"urgent","tags":["t"]}"#.as_bytes().to_vec();
        let created = route(&create, &state);
        assert_eq!(created.status, 201);

        let listing = route(&req("GET", "/api/tasks"), &state);
        assert_eq!(listing.status, 200);
        let json = json::parse(std::str::from_utf8(&listing.body).unwrap()).unwrap();
        assert_eq!(json.get_i64("count"), Some(1));
        let task_json = &json.get_array("tasks").unwrap()[0];
        let id = task_json.get_i64("id").unwrap();

        assert_eq!(route(&req("POST", &format!("/api/tasks/{id}/toggle")), &state).status, 200);
        assert_eq!(route(&req("POST", "/api/tasks/999/done"), &state).status, 404);
        assert_eq!(route(&req("DELETE", &format!("/api/tasks/{id}")), &state).status, 200);
        assert_eq!(route(&req("GET", "/api/tasks"), &state).status, 200);

        // آمار و پاک‌سازی
        assert_eq!(route(&req("GET", "/api/stats"), &state).status, 200);
        assert_eq!(route(&req("POST", "/api/purge"), &state).status, 200);
    }
}
