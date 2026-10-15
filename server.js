const http = require('http');
const fs = require('fs');
const path = require('path');
const { ContinualLearningEngine } = require('./backend/learning-engine');

const PORT = process.env.PORT || 3000;
const HOST = '0.0.0.0';
const PUBLIC_DIR = path.join(__dirname, 'public');

const aiEngine = new ContinualLearningEngine();

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon'
};

function sendJson(res, status, data) {
  res.writeHead(status, {
    'Content-Type': 'application/json; charset=utf-8',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type'
  });
  res.end(JSON.stringify(data));
}

function readJsonBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => {
      body += chunk;
      if (body.length > 5 * 1024 * 1024) req.destroy();
    });
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (e) {
        reject(new Error('فرمت JSON نامعتبر است'));
      }
    });
    req.on('error', reject);
  });
}

const server = http.createServer(async (req, res) => {
  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type'
    });
    return res.end();
  }

  const parsedUrl = new URL(req.url, `http://${req.headers.host || 'localhost'}`);
  const pathname = parsedUrl.pathname;

  try {
    // ۱. وضعیت دیتابیس SQLite و موتور ماشین لرنینگ
    if ((pathname === '/api/status' || pathname === '/api/telemetry') && req.method === 'GET') {
      return sendJson(res, 200, {
        ok: true,
        engine: 'NeuroMount SQLite + ML Continual Engine v2.0',
        telemetry: aiEngine.getDatabaseTelemetry()
      });
    }

    // ۲. پردازش پیام چت و یادگیری خودکار در SQLite
    if (pathname === '/api/chat' && req.method === 'POST') {
      const payload = await readJsonBody(req);
      if (!payload.message) {
        return sendJson(res, 400, { error: 'پیام خالی است' });
      }
      const result = await aiEngine.processChatMessage(payload.message, payload.options || {});
      return sendJson(res, 200, result);
    }

    // ۳. آموزش مستقیم یک گره دانشی جدید در دیتابیس SQLite
    if (pathname === '/api/teach' && req.method === 'POST') {
      const payload = await readJsonBody(req);
      const q = (payload.question || '').trim();
      const a = (payload.answer || '').trim();
      if (!q || !a) {
        return sendJson(res, 400, { error: 'سوال و جواب الزامی است' });
      }
      const nodeId = aiEngine.addKnowledgeNode({
        topic: payload.topic || q.slice(0, 35),
        triggerText: q,
        responseText: a,
        source: 'user-explicit',
        confidence: 3.0
      });
      aiEngine.rebuildMLModels();
      return sendJson(res, 200, {
        ok: true,
        nodeId,
        telemetry: aiEngine.getDatabaseTelemetry()
      });
    }

    // ۴. دانلود و یادگیری زنده یک ریپوزیتوری از GitHub API
    if (pathname === '/api/github/learn' && req.method === 'POST') {
      const payload = await readJsonBody(req);
      if (!payload.repo) {
        return sendJson(res, 400, { error: 'نام ریپوزیتوری گیت‌هاب الزامی است (مثلاً tokio-rs/axum)' });
      }
      const info = await aiEngine.learnFromGitHubRepo(payload.repo);
      return sendJson(res, 200, {
        ok: true,
        repoInfo: info,
        telemetry: aiEngine.getDatabaseTelemetry()
      });
    }

    // ۵. حذف یک رکورد دانشی از دیتابیس SQLite
    if (pathname.startsWith('/api/knowledge/') && req.method === 'DELETE') {
      const id = Number(pathname.split('/').pop());
      if (id) {
        aiEngine.db.prepare('DELETE FROM knowledge_nodes WHERE id = ?').run(id);
        aiEngine.rebuildMLModels();
      }
      return sendJson(res, 200, {
        ok: true,
        telemetry: aiEngine.getDatabaseTelemetry()
      });
    }
  } catch (err) {
    return sendJson(res, 500, { error: err.message || 'خطای سرور' });
  }

  // فایل‌های استاتیک فرانت‌اند
  let safePath = path.normalize(pathname).replace(/^(\.\.[\/\\])+/, '');
  if (safePath === '/' || safePath === '\\') {
    safePath = '/index.html';
  }

  const filePath = path.join(PUBLIC_DIR, safePath);
  if (!filePath.startsWith(PUBLIC_DIR)) {
    res.writeHead(403);
    return res.end('Forbidden');
  }

  fs.stat(filePath, (err, stats) => {
    if (err || !stats.isFile()) {
      const indexFallback = path.join(PUBLIC_DIR, 'index.html');
      fs.readFile(indexFallback, (err2, content) => {
        if (err2) {
          res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
          return res.end('فایل پیدا نشد');
        }
        res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
        res.end(content);
      });
      return;
    }

    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';
    fs.readFile(filePath, (readErr, content) => {
      if (readErr) {
        res.writeHead(500);
        return res.end('Server Error');
      }
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content);
    });
  });
});

server.listen(PORT, HOST, () => {
  console.log(`NeuroMount SQLite + ML Server listening on http://${HOST}:${PORT}`);
});
