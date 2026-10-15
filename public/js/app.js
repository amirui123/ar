/**
 * NeuroMount AI v2.0 — Frontend Controller Connected to Real SQLite + Natural ML + GitHub API
 */

document.addEventListener('DOMContentLoaded', () => {
  const visualNet = new NeuralNetwork([10, 14, 10, 8], {
    activation: 'tanh',
    outputActivation: 'sigmoid'
  });

  let activeStreamTimer = null;
  let isStreaming = false;
  let latestTelemetry = null;

  // المنت‌های اصلی صفحه
  const sidebar = document.getElementById('sidebar');
  const btnOpenSidebar = document.getElementById('btn-open-sidebar');
  const btnCloseSidebar = document.getElementById('btn-close-sidebar');
  const btnNewChat = document.getElementById('btn-new-chat');

  const ghReposCount = document.getElementById('gh-repos-count');
  const ghReposList = document.getElementById('gh-repos-list');
  const sidebarGhForm = document.getElementById('sidebar-gh-form');
  const sidebarGhInput = document.getElementById('sidebar-gh-input');
  const btnGhFetch = document.getElementById('btn-gh-fetch');

  const learnedItemsList = document.getElementById('learned-items-list');
  const learnedCountBadge = document.getElementById('learned-count-badge');

  const sbNodes = document.getElementById('sb-nodes');
  const sbTransitions = document.getElementById('sb-transitions');
  const sbLoss = document.getElementById('sb-loss');
  const headerEpochPill = document.getElementById('header-epoch-pill');

  const themePills = document.querySelectorAll('.theme-pill[data-set-theme]');
  const chatViewport = document.getElementById('chat-viewport');
  const heroWelcome = document.getElementById('hero-welcome');
  const aiOrb = document.getElementById('ai-orb');
  const messagesStream = document.getElementById('messages-stream');
  const suggestionCards = document.querySelectorAll('.suggestion-card');

  const composerContainer = document.getElementById('composer-container');
  const composerForm = document.getElementById('composer-form');
  const composerInput = document.getElementById('composer-input');
  const btnSendMsg = document.getElementById('btn-send-msg');

  const btnToggleTeachMode = document.getElementById('btn-toggle-teach-mode');
  const btnLearnGithubPrompt = document.getElementById('btn-learn-github-prompt');
  const teachModeBanner = document.getElementById('teach-mode-banner');
  const btnCloseTeachMode = document.getElementById('btn-close-teach-mode');
  const composerTeachQ = document.getElementById('composer-teach-q');
  const composerTeachA = document.getElementById('composer-teach-a');
  const btnInjectSynapse = document.getElementById('btn-inject-synapse');

  // کشوی کالبدشکافی
  const brainDrawer = document.getElementById('brain-drawer');
  const drawerBackdrop = document.getElementById('drawer-backdrop');
  const btnOpenBrainDrawer = document.getElementById('btn-open-brain-drawer');
  const btnOpenDrawerTop = document.getElementById('btn-open-drawer-top');
  const btnHeaderBrainXray = document.getElementById('btn-header-brain-xray');
  const btnCloseDrawer = document.getElementById('btn-close-drawer');
  const drawerTabs = document.querySelectorAll('.d-tab');
  const drawerPanels = document.querySelectorAll('.d-panel');

  const llmNeuralCanvas = document.getElementById('llm-neural-canvas');
  const sqliteRecordsTable = document.getElementById('sqlite-records-table');
  const carTrackCanvas = document.getElementById('car-track-canvas');
  const carNeuralCanvas = document.getElementById('car-neural-canvas');
  const btnCarMaster = document.getElementById('btn-car-master');
  const btnCarFastForward = document.getElementById('btn-car-fastforward');
  const btnCarRaw = document.getElementById('btn-car-raw');
  const rustCodeOutput = document.getElementById('rust-code-output');
  const btnCopyRust = document.getElementById('btn-copy-rust');

  function escapeHtml(str) {
    return String(str || '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function formatMarkdownText(raw) {
    let html = escapeHtml(raw);
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
    html = html.replace(/\n/g, '<br/>');
    return html;
  }

  function scrollToBottom() {
    chatViewport.scrollTo({
      top: chatViewport.scrollHeight,
      behavior: 'smooth'
    });
  }

  // =========================================================================
  // ۱. همگام‌سازی زنده با دیتابیس SQLite سرور (/api/telemetry)
  // =========================================================================
  async function refreshTelemetry() {
    try {
      const res = await fetch('/api/telemetry');
      const data = await res.json();
      if (data && data.telemetry) {
        applyTelemetry(data.telemetry);
      }
    } catch (e) {
      console.error('Telemetry error:', e);
    }
  }

  function applyTelemetry(tel) {
    latestTelemetry = tel;
    sbNodes.textContent = tel.nodesCount || 0;
    sbTransitions.textContent = tel.transitionsCount || 0;
    sbLoss.textContent = Number(tel.loss || 0.2).toFixed(2);
    headerEpochPill.textContent = `${tel.nodesCount} گره در SQLite • ${tel.repos?.length || 0} مخزن GitHub`;

    // ۱. لیست مخازن گیت‌هاب در سایدبار
    const repos = tel.repos || [];
    ghReposCount.textContent = repos.length;
    ghReposList.innerHTML = repos
      .map(
        r => `
        <div class="learned-pill gh-repo-pill" data-repo="${escapeHtml(r.repo_full_name)}" title="کلیک کن تا درباره این مخزن بپرسی!">
          🐙 <strong>${escapeHtml(r.repo_full_name)}</strong> (${r.chunks_learned} بخش)
        </div>
      `
      )
      .join('');

    ghReposList.querySelectorAll('.gh-repo-pill').forEach(pill => {
      pill.addEventListener('click', () => {
        const repoName = pill.dataset.repo;
        submitUserMessage(`ریپوزیتوری ${repoName} چیه و چطور کار میکنه؟`);
      });
    });

    // ۲. لیست رکوردهای دیتابیس SQLite در سایدبار
    const records = tel.recentLearned || [];
    learnedCountBadge.textContent = tel.nodesCount || records.length;

    learnedItemsList.innerHTML = records
      .map(rec => {
        const badge = rec.source.startsWith('github:')
          ? '🐙'
          : rec.source === 'curiosity-learned' || rec.source === 'user-explicit' || rec.source === 'auto-profile' || rec.source === 'auto-extracted'
          ? '🎓'
          : '📘';
        return `
          <div class="learned-pill db-node-pill" data-trigger="${escapeHtml(rec.trigger_text.split(' ').slice(0, 4).join(' '))}">
            ${badge} <strong>${escapeHtml(rec.topic)}</strong>
            <div style="font-size:0.68rem;color:var(--text-muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
              ${escapeHtml(rec.response_text.slice(0, 48))}
            </div>
          </div>
        `;
      })
      .join('');

    learnedItemsList.querySelectorAll('.db-node-pill').forEach(pill => {
      pill.addEventListener('click', () => {
        const trig = pill.dataset.trigger;
        if (trig) submitUserMessage(trig);
      });
    });

    // ۳. جدول رکوردهای SQLite داخل کشوی کالبدشکافی
    if (sqliteRecordsTable) {
      sqliteRecordsTable.innerHTML = records
        .map(
          rec => `
          <div style="background:rgba(0,0,0,0.28);border:1px solid var(--border-color);border-radius:9px;padding:8px 10px;font-size:0.76rem;">
            <div style="display:flex;justify-content:space-between;color:var(--accent-secondary);margin-bottom:3px;">
              <strong>#${rec.id} • ${escapeHtml(rec.topic)}</strong>
              <span>منبع: ${escapeHtml(rec.source)}</span>
            </div>
            <div style="color:var(--text-main);">${escapeHtml(rec.response_text.slice(0, 130))}${rec.response_text.length > 130 ? '...' : ''}</div>
          </div>
        `
        )
        .join('');
    }

    if (brainDrawer.classList.contains('open')) {
      renderDrawerVisuals();
    }
  }

  function pulseVisualNet(seedText) {
    const vec = new Array(10).fill(0).map((_, i) => Math.sin((seedText.charCodeAt(i % seedText.length) || 50) * (i + 1) * 0.17));
    const target = new Array(8).fill(0).map((_, i) => (i % 2 === 0 ? 0.85 : 0.15));
    visualNet.trainStep(vec, target, 0.08);
  }

  function renderDrawerVisuals() {
    BrainVisualizer.renderNetwork(llmNeuralCanvas, visualNet, {
      padX: 74,
      padY: 16,
      inputLabels: ['بردار[0]', 'بردار[1]', '3-Gram', 'TF-IDF', 'کسینوسی', 'بیزین', 'توکن ۱', 'توکن ۲', 'بافت', 'تطابق'],
      outputLabels: ['پاسخ SQLite', 'کنجکاوی', 'گیت‌هاب', 'پروفایل', 'تولید متن', 'اطمینان', 'سیناپس', 'حافظه']
    });

    rustCodeOutput.textContent = RustCodeExporter.generateRustBrainCode(
      visualNet,
      `NeuroMount_SQLite_Brain`
    );
  }

  // =========================================================================
  // ۲. ارسال پیام به سرور (/api/chat) با باکس تفکر و استریم کلمه‌به‌کلمه
  // =========================================================================
  function stopCurrentStream() {
    if (activeStreamTimer) {
      clearInterval(activeStreamTimer);
      activeStreamTimer = null;
    }
    isStreaming = false;
    btnSendMsg.classList.remove('streaming');
    document.querySelectorAll('.stream-cursor').forEach(el => el.remove());
    document.querySelectorAll('.ai-avatar.thinking').forEach(el => el.classList.remove('thinking'));
  }

  async function submitUserMessage(text) {
    const clean = text.trim();
    if (!clean) return;

    if (isStreaming) {
      stopCurrentStream();
    }

    heroWelcome.classList.add('hidden');
    pulseVisualNet(clean);

    // ۱. حباب پیام کاربر
    const userRow = document.createElement('div');
    userRow.className = 'msg-row user';
    userRow.innerHTML = `<div class="user-bubble">${formatMarkdownText(clean)}</div>`;
    messagesStream.appendChild(userRow);
    scrollToBottom();

    // ۲. ساخت کارت در حال تفکر هوش مصنوعی
    const aiRow = document.createElement('div');
    aiRow.className = 'msg-row ai';
    aiRow.innerHTML = `
      <div class="ai-message-card">
        <div class="ai-avatar thinking">✦</div>
        <div class="ai-body-col">
          <div class="learned-badge-slot"></div>
          <div class="thinking-box">
            <button type="button" class="thinking-header">
              <div class="thinking-title-wrap">
                <span>🧠</span>
                <span class="thinking-label thinking-shimmer">در حال جستجوی برداری 64D در دیتابیس SQLite و اجرای مدل ML...</span>
              </div>
              <span class="thinking-chevron">▼</span>
            </button>
            <div class="thinking-steps"></div>
          </div>

          <div class="ai-content"><span class="stream-cursor"></span></div>

          <div class="msg-actions-bar">
            <button type="button" class="msg-action-btn btn-copy-reply">📋 کپی</button>
            <button type="button" class="msg-action-btn teach-highlight btn-open-inline-teach">
              🎓 ثبت/اصلاح پاسخ در دیتابیس SQLite
            </button>
            <button type="button" class="msg-action-btn btn-toggle-probs">
              📊 جزئیات بازیابی و احتمال توکن‌ها
            </button>
          </div>

          <div class="inline-teach-popover">
            <label>🎓 پاسخ درست برای «${escapeHtml(clean.slice(0, 45))}» را بنویس تا مستقیم در SQLite ذخیره شود:</label>
            <div class="inline-teach-row">
              <input type="text" class="inline-teach-input" placeholder="پاسخ دقیق را بنویسید..." />
              <button type="button" class="btn-submit-inline-teach">ذخیره در SQLite ⚡</button>
            </div>
          </div>

          <div class="inline-probs-box"></div>
        </div>
      </div>
    `;

    messagesStream.appendChild(aiRow);
    scrollToBottom();

    const avatarEl = aiRow.querySelector('.ai-avatar');
    const badgeSlot = aiRow.querySelector('.learned-badge-slot');
    const thinkingBox = aiRow.querySelector('.thinking-box');
    const thinkingHeader = aiRow.querySelector('.thinking-header');
    const thinkingLabel = aiRow.querySelector('.thinking-label');
    const thinkingStepsEl = aiRow.querySelector('.thinking-steps');
    const contentEl = aiRow.querySelector('.ai-content');
    const actionsBar = aiRow.querySelector('.msg-actions-bar');
    const inlineTeachPopover = aiRow.querySelector('.inline-teach-popover');
    const inlineTeachInput = aiRow.querySelector('.inline-teach-input');
    const btnSubmitInlineTeach = aiRow.querySelector('.btn-submit-inline-teach');
    const inlineProbsBox = aiRow.querySelector('.inline-probs-box');

    thinkingHeader.addEventListener('click', () => {
      thinkingBox.classList.toggle('open');
    });

    isStreaming = true;
    btnSendMsg.classList.add('streaming');

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: clean })
      });
      const result = await response.json();

      if (result.dbStats) {
        applyTelemetry(result.dbStats);
      }

      if (result.isLearnedEvent) {
        badgeSlot.innerHTML = `<div class="learned-celebration-badge">⚡ دانش جدید در دیتابیس SQLite ذخیره و مدل Bayes بازآموزی شد!</div>`;
      } else if (result.isCuriosityPrompt) {
        badgeSlot.innerHTML = `<div class="learned-celebration-badge" style="background:rgba(245,158,11,0.15);border-color:rgba(245,158,11,0.45);color:#fcd34d;">🤔 حلقه کنجکاوی فعال: پیام بعدی شما خودکار به عنوان پاسخ این سوال یاد گرفته می‌شود!</div>`;
      }

      thinkingStepsEl.innerHTML = (result.thinkingSteps || [])
        .map(s => `<div class="t-step">${escapeHtml(s)}</div>`)
        .join('');

      thinkingLabel.classList.remove('thinking-shimmer');
      thinkingLabel.textContent = `پردازش تکمیل شد (${result.matchedSource || 'SQLite ML'} • اطمینان ${result.confidence || 90}%)`;

      const probsHtml = (result.nextTokenPredictions || [])
        .map(item => {
          const pct = (item.prob * 100).toFixed(1);
          return `
            <div class="prob-mini-row">
              <span><strong>${escapeHtml(item.token)}</strong></span>
              <div class="prob-mini-track">
                <div class="prob-mini-fill" style="width:${Math.max(5, pct)}%"></div>
              </div>
              <span style="font-family:monospace;text-align:left">${pct}%</span>
            </div>
          `;
        })
        .join('');

      inlineProbsBox.innerHTML = `
        <div style="font-size:0.74rem;color:var(--text-muted);margin-bottom:4px;">
          منبع تصمیم: <strong>${escapeHtml(result.matchedSource || '')}</strong> | توکن‌های بعدی در گراف SQLite:
        </div>
        ${probsHtml || '<div style="font-size:0.72rem;color:var(--text-muted)">در حال تکمیل گراف توکن...</div>'}
      `;

      // استریم نرم کلمه‌به‌کلمه
      const replyText = result.reply || 'پاسخی دریافت نشد.';
      const tokensToStream = replyText.split(/(\s+)/);
      let idx = 0;
      let accumulated = '';

      activeStreamTimer = setInterval(() => {
        const chunkCount = Math.random() > 0.55 ? 2 : 1;
        for (let c = 0; c < chunkCount && idx < tokensToStream.length; c++, idx++) {
          accumulated += tokensToStream[idx];
        }

        contentEl.innerHTML = `${formatMarkdownText(accumulated)}<span class="stream-cursor"></span>`;
        scrollToBottom();

        if (idx >= tokensToStream.length) {
          stopCurrentStream();
          contentEl.innerHTML = formatMarkdownText(accumulated);
          avatarEl.classList.remove('thinking');
          actionsBar.classList.add('visible');
        }
      }, 24);

      // رویداد دکمه‌های زیر پیام
      aiRow.querySelector('.btn-copy-reply').addEventListener('click', e => {
        navigator.clipboard.writeText(replyText);
        e.currentTarget.textContent = '✅ کپی شد';
        setTimeout(() => {
          e.currentTarget.textContent = '📋 کپی';
        }, 1500);
      });

      aiRow.querySelector('.btn-open-inline-teach').addEventListener('click', () => {
        inlineTeachPopover.classList.toggle('open');
        if (inlineTeachPopover.classList.contains('open')) inlineTeachInput.focus();
      });

      aiRow.querySelector('.btn-toggle-probs').addEventListener('click', () => {
        inlineProbsBox.classList.toggle('open');
      });

      btnSubmitInlineTeach.addEventListener('click', async () => {
        const newAns = inlineTeachInput.value.trim();
        if (!newAns) return;
        await fetch('/api/teach', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ question: clean, answer: newAns })
        });
        inlineTeachPopover.classList.remove('open');
        await refreshTelemetry();
        submitUserMessage(clean);
      });
    } catch (err) {
      stopCurrentStream();
      contentEl.innerHTML = `⚠️ خطا در ارتباط با سرور: ${escapeHtml(err.message)}`;
    }
  }

  // =========================================================================
  // ۳. رویدادهای کامپوزر، فرم گیت‌هاب و آموزش مستقیم
  // =========================================================================
  composerInput.addEventListener('input', () => {
    composerInput.style.height = 'auto';
    composerInput.style.height = `${Math.min(160, composerInput.scrollHeight)}px`;
    aiOrb.classList.toggle('typing-active', composerInput.value.trim().length > 0);
  });

  composerInput.addEventListener('keydown', e => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      composerForm.requestSubmit();
    }
  });

  composerForm.addEventListener('submit', e => {
    e.preventDefault();
    if (isStreaming) {
      stopCurrentStream();
      return;
    }
    const text = composerInput.value.trim();
    if (!text) return;
    composerInput.value = '';
    composerInput.style.height = 'auto';
    aiOrb.classList.remove('typing-active');
    submitUserMessage(text);
  });

  function toggleTeachBanner(forceState) {
    const isOpen = forceState !== undefined ? forceState : !teachModeBanner.classList.contains('open');
    teachModeBanner.classList.toggle('open', isOpen);
    btnToggleTeachMode.classList.toggle('active', isOpen);
    composerContainer.classList.toggle('teach-active', isOpen);
    if (isOpen) composerTeachQ.focus();
  }

  btnToggleTeachMode.addEventListener('click', () => toggleTeachBanner());
  btnCloseTeachMode.addEventListener('click', () => toggleTeachBanner(false));

  btnInjectSynapse.addEventListener('click', async () => {
    const q = composerTeachQ.value.trim();
    const a = composerTeachA.value.trim();
    if (!q || !a) return;
    composerTeachQ.value = '';
    composerTeachA.value = '';
    toggleTeachBanner(false);
    submitUserMessage(`یاد بگیر: وقتی گفتم ${q} بگو ${a}`);
  });

  btnLearnGithubPrompt.addEventListener('click', () => {
    composerInput.value = 'از گیت‌هاب یاد بگیر: ';
    composerInput.focus();
  });

  // فرم دانلود ریپوزیتوری گیت‌هاب در سایدبار
  sidebarGhForm.addEventListener('submit', e => {
    e.preventDefault();
    const repo = sidebarGhInput.value.trim();
    if (!repo) return;
    sidebarGhInput.value = '';
    submitUserMessage(`از گیت‌هاب یاد بگیر: ${repo}`);
  });

  suggestionCards.forEach(card => {
    card.addEventListener('click', () => {
      const prompt = card.dataset.prompt;
      if (prompt) submitUserMessage(prompt);
    });
  });

  // سایدبار، گفتگوی جدید و تغییر تم
  btnOpenSidebar.addEventListener('click', () => sidebar.classList.remove('collapsed'));
  btnCloseSidebar.addEventListener('click', () => sidebar.classList.add('collapsed'));

  btnNewChat.addEventListener('click', () => {
    stopCurrentStream();
    messagesStream.innerHTML = '';
    heroWelcome.classList.remove('hidden');
  });

  themePills.forEach(pill => {
    pill.addEventListener('click', () => {
      themePills.forEach(p => p.classList.remove('active'));
      pill.classList.add('active');
      document.documentElement.setAttribute('data-theme', pill.dataset.setTheme);
    });
  });

  // کشوی کالبدشکافی و پیست خودران
  const carLab = new AutonomousCarLab(carTrackCanvas, tel => {
    if (brainDrawer.classList.contains('open')) {
      BrainVisualizer.renderNetwork(carNeuralCanvas, tel.leadBrain, {
        padX: 74,
        padY: 14,
        inputLabels: ['چپ -75°', 'چپ -45°', 'چپ -20°', 'روبرو 0°', 'راست +20°', 'راست +45°', 'راست +75°', 'سرعت'],
        outputLabels: ['فرمان', 'سرعت']
      });
    }
  });

  function openDrawer() {
    brainDrawer.classList.add('open');
    drawerBackdrop.classList.add('open');
    renderDrawerVisuals();
  }

  function closeDrawer() {
    brainDrawer.classList.remove('open');
    drawerBackdrop.classList.remove('open');
    carLab.isRunning = false;
  }

  btnOpenBrainDrawer.addEventListener('click', openDrawer);
  btnOpenDrawerTop.addEventListener('click', openDrawer);
  btnHeaderBrainXray.addEventListener('click', openDrawer);
  btnCloseDrawer.addEventListener('click', closeDrawer);
  drawerBackdrop.addEventListener('click', closeDrawer);

  drawerTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      drawerTabs.forEach(t => t.classList.remove('active'));
      drawerPanels.forEach(p => p.classList.remove('active'));
      tab.classList.add('active');
      const panel = document.getElementById(tab.dataset.dtab);
      if (panel) panel.classList.add('active');

      if (tab.dataset.dtab === 'dtab-car') {
        carLab.isRunning = true;
        carLab.render();
      } else {
        carLab.isRunning = false;
      }
      renderDrawerVisuals();
    });
  });

  btnCarMaster.addEventListener('click', () => carLab.mountMasterBrain());
  btnCarFastForward.addEventListener('click', () => carLab.fastForwardGenerations(8));
  btnCarRaw.addEventListener('click', () => carLab.resetToRawBrain());

  btnCopyRust.addEventListener('click', () => {
    navigator.clipboard.writeText(rustCodeOutput.textContent);
    btnCopyRust.textContent = '✅ کپی شد!';
    setTimeout(() => {
      btnCopyRust.textContent = '📋 کپی کد Rust';
    }, 1500);
  });

  function carLoop() {
    if (carLab.isRunning) {
      carLab.stepPhysics();
      carLab.render();
      if (carLab.frameCounter % 5 === 0) {
        carLab.onUpdate(carLab.getTelemetry());
      }
    }
    requestAnimationFrame(carLoop);
  }

  refreshTelemetry();
  carLoop();
});
