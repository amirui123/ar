/**
 * NeuroMount Live-Learning Mini-LLM Engine (v2 - ChatGPT / Claude Grade Conversational & Self-Learning Core)
 * موتور مدل زبانی خودآموز با:
 * ۱. شبکه عصبی واقعی (Forward + Backpropagation)
 * ۲. ماتریس توجه چندمرتبه‌ای (Multi-Order Contextual Logits با گرادیان Cross-Entropy)
 * ۳. یادگیری مستمر محاوره‌ای (تشخیص خودکار آموزش در چت + حافظه سیناپسی پایدار در مرورگر)
 * ۴. تولید زنجیره تفکر (Chain-of-Thought Neural Trace) مشابه Claude / ChatGPT o1
 */

const LLM_PRESET_CORPUSES = {
  ai_rust: {
    title: '🦀 هوش مصنوعی، Rust و برنامه‌نویسی',
    description: 'دستیار هوشمند مسلط به ماشین لرنینگ، شبکه عصبی، زبان Rust و توسعه وب',
    text: `هوش مصنوعی با یادگیری ماشین می‌تواند الگوهای پیچیده را از داده‌ها کشف کند و با هر تجربه جدید هوشمندتر شود.
شبکه عصبی از لایه‌های نورون و وزن‌های سیناپسی تشکیل شده است که با الگوریتم انتشار برگشتی خطا آموزش می‌بینند.
زبان راست یک زبان برنامه‌نویسی سیستمی، فوق‌العاده سریع و امن در مدیریت حافظه است که بدون Garbage Collector کار می‌کند.
ترکیب زبان راست با پایتون و جاوااسکریپت از طریق WebAssembly و PyO3 قدرت پردازش هوش مصنوعی را چند برابر می‌کند.
مدل زبانی بزرگ با پیش‌بینی توکن بعدی بر اساس توزیع احتمال و مکانیزم توجه متن تولید می‌کند.
وقتی خطای مدل در هر دور آموزش کاهش می‌یابد، پیش‌بینی کلمات دقیق‌تر، روان‌تر و هوشمندتر می‌شود.
برای ساخت سایت سریع و امن می‌توان از فریم‌ورک Axum یا Actix در زبان راست در کنار فرانت‌اند مدرن استفاده کرد.
یادگیری تقویتی به ایجنت اجازه می‌دهد با آزمون و خطا در محیط واقعی بهترین تصمیم را یاد بگیرد.
سوال: سلام | جواب: سلام داداش! خیلی خوش اومدی. من هوش مصنوعی خودآموز تو هستم؛ هم می‌تونیم گپ بزنیم و هم هر چیزی بهم بگی رو توی شبکه عصبیم یاد می‌گیرم!
سوال: سلام چطوری | جواب: سلام رفیق جان! عالی‌ام، سیناپس‌هام آماده یادگیریه! تو چطوری؟ امروز می‌خوای روی چه ایده‌ای کار کنیم؟
سوال: خوبی | جواب: قربونت داداش، من عالیم! هر چقدر بیشتر باهام حرف بزنی و بهم دیتا بدی، شبکه عصبیم قوی‌تر هم میشه.
سوال: اسمت چیه | جواب: اسم من **نورومانت (NeuroMount)** هست؛ یه مدل زبانی و شبکه عصبی خودآموز که مستقیم جلوی چشمت یاد می‌گیره و رشد می‌کنه.
سوال: تو کی هستی | جواب: من **NeuroMount AI** هستم؛ یه هوش مصنوعی با قابلیت یادگیری زنده (Continual Machine Learning) که می‌تونی مغزم رو آموزش بدی یا روی ایجنت‌های مختلف سوار کنی.
سوال: زبان راست چیه | جواب: زبان **Rust (راست)** یه زبان برنامه‌نویسی فوق‌العاده سریع، مدرن و امنه که بدون نیاز به Garbage Collector سرعتی هم‌رده C++ داره و برای وب، بلاکچین و موتورهای هوش مصنوعی عالیه.
سوال: آیا راست برای ساخت سایت خوبه | جواب: آره صد در صد! برای بک‌اند می‌تونی از فریم‌ورک‌های فوق‌سریع **Axum** یا **Actix-web** استفاده کنی و برای فرانت‌اند هم با **WebAssembly** یا ترکیبش با React و Next.js یه سایت ضدگلوله بسازی.
سوال: چطوری راست رو با پایتون ترکیب کنم | جواب: بهترین راه استفاده از کتابخونه **PyO3** و ابزار **maturin** هست! هسته سنگین پردازشی یا ماشین لرنینگ رو با Rust می‌نویسی و مستقیم مثل یه پکیج عادی داخل پایتون ایمپورت می‌کنی.
سوال: چطوری یاد میگیری | جواب: من از دو طریق یاد می‌گیرم:\n۱. **گرادیان کاهشی (Backpropagation):** با خوندن متن‌ها و تنظیم وزن‌های شبکه عصبی در هر Epoch.\n۲. **آموزش مستقیم در چت:** کافیه توی چت بنویسی «یاد بگیر: وقتی گفتم X بگو Y» یا دکمه «آموزش به مغز» رو بزنی تا همون لحظه توی حافظه سیناپسیم ثبتش کنم!
سوال: ماشین لرنینگ چیه | جواب: **ماشین لرنینگ (یادگیری ماشین)** یعنی به جای اینکه قوانین رو خط‌به‌خط برای کامپیوتر کدنویسی کنیم، یک شبکه عصبی بسازیم که خودش از روی داده‌ها و تجربه‌ها الگوها رو کشف کنه و یاد بگیره.
سوال: چه کمکی میتونی بکنی | جواب: می‌تونم باهات گفتگو کنم، متن و ایده تولید کنم، چیزهای جدیدی که بهم یاد میدی رو توی شبکه عصبیم ذخیره کنم، احتمال کلمات بعدی رو نشونت بدم و کد شبکه عصبی رو به زبان **Rust** خروجی بدم!`
  },
  friendly_chat: {
    title: '💬 مکالمه صمیمی و روزمره',
    description: 'سبک گفتگوی خودمانی، شوخ‌طبع و همراه',
    text: `سلام داداش خوش اومدی به فضای اختصاصی خودت!
اینجا هر متنی یا دیالوگی به من بدی، همون لحظه توی شبکه عصبی خودم ذخیره‌ش می‌کنم و یاد می‌گیرم.
برنامه‌نویسی یکی از جذاب‌ترین مهارت‌های دنیاست چون می‌تونی ایده‌هات رو به واقعیت تبدیل کنی.
هر چقدر تعداد دورهای آموزش بیشتر بشه، خطای شبکه عصبی کمتر میشه و جملات من طبیعی‌تر میشه.
سوال: سلام داداش | جواب: سلام عزیز دل! چطوری؟ چه خبر؟ بگو ببینم امروز تو چه فکری هستی!
سوال: چطوری | جواب: مخلصم داداش! حالم توپه، مخصوصاً وقتی سیناپس‌های مغزم رو آپدیت می‌کنی!
سوال: دمت گرم | جواب: فدایی داری داداش! هر کاری داشتی یا خواستی چیز جدیدی یادم بدی فقط اشاره کن.
سوال: یه جک بگو | جواب: یه برنامه‌نویس Rust میره کافه، گارسون میگه چی میل دارین؟ میگه هر چی میاری بیار فقط وسطش با Borrow Checker دعوام نشه! 😄🦀`
  },
  persian_literature: {
    title: '📜 ادبیات و حکمت پارسی',
    description: 'سخنوری ادبی، اشعار کلاسیک و جملات حکیمانه',
    text: `توانا بود هر که دانا بود ز دانش دل پیر برنا بود.
درخت تو گر بار دانش بگیرد به زیر آوری چرخ نیلوفری را.
خرد رهنمای و خرد دلگشای خرد دست گیرد به هر دو سرای.
بنی آدم اعضای یک پیکرند که در آفرینش ز یک گوهرند.
چو عضوی به درد آورد روزگار دگر عضوها را نماند قرار.
دانش چراغ روشن راه زندگی است و اندیشه نیک انسان را به اوج می‌رساند.
سوال: شعر بخوان | جواب: بنی آدم اعضای یک پیکرند / که در آفرینش ز یک گوهرند\nچو عضوی به درد آورد روزگار / دگر عضوها را نماند قرار ✨
سوال: حکمت دانش چیست | جواب: فردوسی بزرگ می‌فرماید: «توانا بود هر که دانا بود، ز دانش دل پیر برنا بود.» دانش چراغ فروزان خرد است.`
  }
};

class MiniLLM {
  constructor() {
    this.vocab = ['<PAD>', '<BOS>', '<EOS>'];
    this.wordToIdx = new Map();
    this.idxToWord = new Map();

    // شبکه عصبی هسته: [10 -> 14 -> 10 -> 8]
    this.neuralCore = new NeuralNetwork([10, 14, 10, 8], {
      activation: 'tanh',
      outputActivation: 'sigmoid'
    });

    this.contextLogits = new Map();
    this.unigramCounts = new Map();
    this.qaPairs = [];
    this.customFacts = []; // حقایق و نکاتی که کاربر در چت یاد داده است
    this.sentences = [];
    this.trainingPairs = [];

    this.epoch = 0;
    this.lossHistory = [];
    this.accuracyHistory = [];
    this.evolutionSnapshots = [];
    this.lastLoss = 4.65;
    this.lastAccuracy = 4.0;

    this.rebuildIndexMaps();
  }

  rebuildIndexMaps() {
    this.wordToIdx.clear();
    this.idxToWord.clear();
    this.vocab.forEach((w, i) => {
      this.wordToIdx.set(w, i);
      this.idxToWord.set(i, w);
    });
  }

  normalizeText(text) {
    if (!text) return '';
    return text
      .replace(/ي/g, 'ی')
      .replace(/ك/g, 'ک')
      .replace(/\u200c/g, ' ')
      .trim();
  }

  tokenize(text) {
    if (!text) return [];
    const cleaned = this.normalizeText(text)
      .replace(/([؟?!.،,:؛])/g, ' $1 ')
      .replace(/\s+/g, ' ')
      .trim();
    if (!cleaned) return [];
    return cleaned.split(' ').filter(Boolean);
  }

  prepareCorpus(rawText, resetModel = false) {
    if (resetModel) {
      this.vocab = ['<PAD>', '<BOS>', '<EOS>'];
      this.rebuildIndexMaps();
      this.contextLogits.clear();
      this.unigramCounts.clear();
      this.qaPairs = [];
      this.epoch = 0;
      this.lossHistory = [];
      this.accuracyHistory = [];
      this.evolutionSnapshots = [];
      this.neuralCore.initWeights();
    }

    const lines = rawText
      .split(/\n+/)
      .map(l => l.trim())
      .filter(Boolean);

    this.sentences = [];

    for (const line of lines) {
      if (line.includes('|') && (line.includes('سوال:') || line.includes('پرسش:'))) {
        const parts = line.split('|');
        const q = parts[0].replace(/^(سوال|پرسش)\s*:\s*/i, '').trim();
        const a = parts.slice(1).join('|').replace(/^(جواب|پاسخ)\s*:\s*/i, '').trim();
        if (q && a) {
          this.addQAPair(q, a);
          this.sentences.push(`${q} ${a}`);
          this.sentences.push(a);
          continue;
        }
      }
      this.sentences.push(line);
    }

    for (const sent of this.sentences) {
      const tokens = this.tokenize(sent);
      for (const tok of tokens) {
        if (!this.wordToIdx.has(tok)) {
          const idx = this.vocab.length;
          this.vocab.push(tok);
          this.wordToIdx.set(tok, idx);
          this.idxToWord.set(idx, tok);
        }
        const idx = this.wordToIdx.get(tok);
        this.unigramCounts.set(idx, (this.unigramCounts.get(idx) || 0) + 1);
      }
    }

    this.trainingPairs = [];
    for (const sent of this.sentences) {
      const tokens = ['<BOS>', '<BOS>', ...this.tokenize(sent), '<EOS>'];
      for (let i = 2; i < tokens.length; i++) {
        const w_2 = tokens[i - 2];
        const w_1 = tokens[i - 1];
        const target = tokens[i];
        const targetIdx = this.wordToIdx.get(target);

        const c1 = w_1;
        const c2 = `${w_2} ${w_1}`;
        const c3 = i >= 3 ? `${tokens[i - 3]} ${w_2} ${w_1}` : null;

        for (const ctxKey of [c1, c2, c3]) {
          if (!ctxKey) continue;
          if (!this.contextLogits.has(ctxKey)) {
            this.contextLogits.set(ctxKey, new Map());
          }
          const map = this.contextLogits.get(ctxKey);
          if (!map.has(targetIdx)) {
            map.set(targetIdx, (Math.random() - 0.5) * 0.2);
          }
        }

        const features = this.encodeTokensToFeatures([w_2, w_1]);
        const targetVec = this.encodeTokenToTargetVec(targetIdx);

        this.trainingPairs.push({
          c1,
          c2,
          c3,
          targetIdx,
          features,
          targetVec
        });
      }
    }

    if (this.epoch === 0) {
      this.evaluateCorpusMetrics();
    }
  }

  addQAPair(question, answer, isUserTaught = false) {
    const cleanQ = this.normalizeText(question).replace(/[؟?!.،]/g, '').trim();
    const qTokens = this.tokenize(cleanQ);
    const existing = this.qaPairs.find(p => p.cleanQ === cleanQ);
    if (existing) {
      existing.answer = answer;
      existing.qTokens = qTokens;
      existing.isUserTaught = existing.isUserTaught || isUserTaught;
      existing.learnedStrength = Math.min(8.0, (existing.learnedStrength || 1.0) + (isUserTaught ? 2.5 : 0.6));
    } else {
      this.qaPairs.unshift({
        question: question.trim(),
        cleanQ,
        answer: answer.trim(),
        qTokens,
        isUserTaught,
        learnedStrength: isUserTaught ? 3.5 : 1.0
      });
    }
  }

  encodeTokensToFeatures(tokens) {
    const vec = new Array(10).fill(0);
    tokens.forEach((tok, pos) => {
      let hash = 0;
      for (let i = 0; i < tok.length; i++) {
        hash = (hash * 31 + tok.charCodeAt(i)) % 997;
      }
      for (let d = 0; d < 5; d++) {
        const idx = (pos * 5 + d) % 10;
        vec[idx] = Math.sin(hash * (d + 1) * 0.13);
      }
    });
    return vec;
  }

  encodeTokenToTargetVec(tokenIdx) {
    const vec = new Array(8).fill(0.1);
    vec[tokenIdx % 8] = 0.9;
    vec[(tokenIdx * 3) % 8] = 0.85;
    return vec;
  }

  trainOneEpoch(lr = 0.35) {
    if (this.trainingPairs.length === 0) return { loss: 0, accuracy: 0 };

    let totalCrossEntropy = 0;
    let correctPredictions = 0;

    const indices = this.trainingPairs.map((_, idx) => idx);
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [indices[i], indices[j]] = [indices[j], indices[i]];
    }

    const neuralSampleStep = Math.max(1, Math.floor(indices.length / 16));

    for (let step = 0; step < indices.length; step++) {
      const sample = this.trainingPairs[indices[step]];
      const { c1, c2, c3, targetIdx, features, targetVec } = sample;

      if (step % neuralSampleStep === 0) {
        this.neuralCore.trainStep(features, targetVec, Math.min(0.08, lr * 0.2));
      }

      const candidates = new Set([targetIdx]);
      const contexts = [
        { key: c3, scale: 1.8 },
        { key: c2, scale: 1.3 },
        { key: c1, scale: 0.8 }
      ].filter(c => c.key && this.contextLogits.has(c.key));

      for (const ctx of contexts) {
        for (const candIdx of this.contextLogits.get(ctx.key).keys()) {
          candidates.add(candIdx);
        }
      }

      const vocabLen = this.vocab.length;
      for (let n = 0; n < 4; n++) {
        candidates.add(Math.floor(Math.random() * vocabLen));
      }

      const candList = Array.from(candidates);
      const logits = new Array(candList.length).fill(0);

      for (let i = 0; i < candList.length; i++) {
        const tokIdx = candList[i];
        let score = 0;
        for (const ctx of contexts) {
          const map = this.contextLogits.get(ctx.key);
          score += (map.get(tokIdx) || 0) * ctx.scale;
        }
        logits[i] = score;
      }

      const probs = NeuralNetwork.softmax(logits, 1.0);
      const targetPos = candList.indexOf(targetIdx);
      const targetProb = Math.max(1e-6, probs[targetPos]);

      totalCrossEntropy += -Math.log(targetProb);

      let bestPos = 0;
      for (let i = 1; i < probs.length; i++) {
        if (probs[i] > probs[bestPos]) bestPos = i;
      }
      if (candList[bestPos] === targetIdx) {
        correctPredictions++;
      }

      for (let i = 0; i < candList.length; i++) {
        const tokIdx = candList[i];
        const y = i === targetPos ? 1.0 : 0.0;
        const grad = probs[i] - y;

        for (const ctx of contexts) {
          const map = this.contextLogits.get(ctx.key);
          const currentW = map.get(tokIdx) || 0;
          const updatedW = currentW - lr * ctx.scale * grad;
          if (Math.abs(updatedW) > 0.005 || tokIdx === targetIdx) {
            map.set(tokIdx, updatedW);
          }
        }
      }
    }

    for (const qa of this.qaPairs) {
      qa.learnedStrength = Math.min(10.0, (qa.learnedStrength || 1.0) + 0.15);
    }

    this.epoch++;
    const avgLoss = totalCrossEntropy / this.trainingPairs.length;
    const accuracy = (correctPredictions / this.trainingPairs.length) * 100;

    this.lastLoss = avgLoss;
    this.lastAccuracy = accuracy;
    this.lossHistory.push({ epoch: this.epoch, loss: avgLoss, accuracy });

    if (
      this.epoch === 1 ||
      this.epoch === 5 ||
      this.epoch === 15 ||
      this.epoch === 30 ||
      this.epoch % 25 === 0
    ) {
      const sampleText = this.generateText('', 14, Math.max(0.25, 1.1 - this.epoch * 0.015));
      this.evolutionSnapshots.unshift({
        epoch: this.epoch,
        loss: avgLoss.toFixed(3),
        accuracy: accuracy.toFixed(1),
        text: sampleText
      });
      if (this.evolutionSnapshots.length > 8) {
        this.evolutionSnapshots.pop();
      }
    }

    return { epoch: this.epoch, loss: avgLoss, accuracy };
  }

  evaluateCorpusMetrics() {
    if (this.trainingPairs.length === 0) {
      this.lastLoss = 4.5;
      this.lastAccuracy = 0;
      return;
    }
    this.lastLoss = Math.log(Math.max(10, this.vocab.length));
    this.lastAccuracy = (1 / Math.max(10, this.vocab.length)) * 100;
  }

  predictNextTokenDistribution(promptText, temperature = 0.7) {
    const tokens = ['<BOS>', '<BOS>', ...this.tokenize(promptText)];
    const w_1 = tokens[tokens.length - 1];
    const w_2 = tokens[tokens.length - 2];
    const w_3 = tokens.length >= 3 ? tokens[tokens.length - 3] : null;

    const features = this.encodeTokensToFeatures([w_2, w_1]);
    this.neuralCore.forward(features);

    const c1 = w_1;
    const c2 = `${w_2} ${w_1}`;
    const c3 = w_3 ? `${w_3} ${w_2} ${w_1}` : null;

    const scoreMap = new Map();
    const contexts = [
      { key: c3, scale: 2.2 },
      { key: c2, scale: 1.5 },
      { key: c1, scale: 0.9 }
    ];

    for (const ctx of contexts) {
      if (ctx.key && this.contextLogits.has(ctx.key)) {
        const map = this.contextLogits.get(ctx.key);
        for (const [tokIdx, weight] of map.entries()) {
          if (tokIdx === 0 || tokIdx === 1) continue;
          scoreMap.set(tokIdx, (scoreMap.get(tokIdx) || 0) + weight * ctx.scale);
        }
      }
    }

    if (scoreMap.size < 6) {
      const vocabLen = this.vocab.length;
      for (let idx = 2; idx < Math.min(vocabLen, 45); idx++) {
        if (!scoreMap.has(idx)) {
          const baseNoise = this.epoch === 0 ? (Math.random() - 0.5) * 1.2 : -1.5 + Math.random() * 0.2;
          scoreMap.set(idx, baseNoise);
        }
      }
    }

    const entries = Array.from(scoreMap.entries());
    const logits = entries.map(e => e[1]);
    const probs = NeuralNetwork.softmax(logits, temperature);

    const distribution = entries.map((entry, i) => ({
      tokenIdx: entry[0],
      token: this.idxToWord.get(entry[0]) || '?',
      logit: entry[1],
      prob: probs[i]
    }));

    distribution.sort((a, b) => b.prob - a.prob);
    return distribution;
  }

  sampleNextToken(promptText, temperature = 0.7) {
    const dist = this.predictNextTokenDistribution(promptText, temperature);
    if (dist.length === 0) return '<EOS>';

    const topK = dist.slice(0, Math.min(8, dist.length));
    const sumProb = topK.reduce((acc, item) => acc + item.prob, 0);
    let r = Math.random() * sumProb;

    for (const item of topK) {
      r -= item.prob;
      if (r <= 0) return item.token;
    }
    return topK[0].token;
  }

  generateText(seedPrompt = '', maxTokens = 18, temperature = 0.65) {
    if (this.vocab.length <= 3) {
      return 'مدل هنوز دیتایی دریافت نکرده است.';
    }

    let currentTokens = this.tokenize(seedPrompt);
    if (currentTokens.length === 0 && this.sentences.length > 0) {
      const randSent = this.sentences[Math.floor(Math.random() * this.sentences.length)];
      const firstToks = this.tokenize(randSent);
      if (firstToks.length > 0) {
        currentTokens = [firstToks[0]];
      }
    }

    for (let step = 0; step < maxTokens; step++) {
      const currentText = currentTokens.join(' ');
      const nextTok = this.sampleNextToken(currentText, temperature);
      if (nextTok === '<EOS>') {
        if (currentTokens.length >= 5) break;
        continue;
      }
      currentTokens.push(nextTok);
    }

    return currentTokens
      .join(' ')
      .replace(/\s+([؟?!.،,:؛])/g, '$1')
      .trim();
  }

  /**
   * بررسی آیا پیام کاربر یک دستور آموزش مستقیم در چت است؟
   * مثال‌ها:
   * - «یاد بگیر: وقتی گفتم X بگو Y»
   * - «یاد بگیر هر وقت پرسیدم X جواب بده Y»
   * - «سوال: X | جواب: Y»
   * - «یاد بگیر که ...»
   */
  parseNaturalTeachingCommand(text) {
    const trimmed = text.trim();

    // ۱. فرمت «سوال: ... | جواب: ...»
    if (trimmed.includes('|')) {
      const parts = trimmed.split('|');
      const q = parts[0].replace(/^(یاد بگیر\s*:?\s*)?(سوال|پرسش|وقتی گفتم)\s*:?\s*/i, '').trim();
      const a = parts.slice(1).join('|').replace(/^(جواب|پاسخ|بگو)\s*:?\s*/i, '').trim();
      if (q && a) return { type: 'qa', question: q, answer: a };
    }

    // ۲. فرمت «وقتی گفتم ... بگو ...» یا «هر وقت پرسیدم ... بگو ...»
    const condMatch = trimmed.match(/^(?:یاد بگیر\s*:?\s*)?(?:هر وقت|وقتی)\s+(?:گفتم|پرسیدم)\s+(.+?)\s+(?:بگو|جواب بده)\s+(.+)$/i);
    if (condMatch) {
      return {
        type: 'qa',
        question: condMatch[1].replace(/[«»"']/g, '').trim(),
        answer: condMatch[2].replace(/[«»"']/g, '').trim()
      };
    }

    // ۳. فرمت «یاد بگیر: ...» یا «یاد بگیر که ...» (یادگیری حقیقت یا جمله جدید)
    const factMatch = trimmed.match(/^یاد بگیر(?:\s+که)?\s*:?\s+(.+)$/i);
    if (factMatch && factMatch[1].length > 3) {
      const content = factMatch[1].trim();
      // اگر داخلش «یعنی» یا «هست» بود، هم به عنوان جمله و هم Q&A ذخیره کن
      const isMatch = content.match(/^(.+?)\s+(?:یعنی|همون|عبارت است از|اسمش)\s+(.+)$/);
      if (isMatch) {
        return {
          type: 'qa',
          question: isMatch[1].trim(),
          answer: content
        };
      }
      return {
        type: 'fact',
        fact: content
      };
    }

    return null;
  }

  /**
   * پاسخگویی هوشمند به همراه تولید زنجیره تفکر (Thinking Steps) مشابه Claude و ChatGPT
   */
  chatReply(userMessage, temperature = 0.55) {
    const cleanMsg = this.normalizeText(userMessage).replace(/[؟?!.،]/g, '').trim();
    const userTokens = this.tokenize(cleanMsg);

    const features = this.encodeTokensToFeatures(userTokens.slice(-2));
    const activations = this.neuralCore.forward(features);
    const activeNeuronsCount = activations.filter(v => v > 0.5).length;

    // ۱. بررسی آیا کاربر در حال یاد دادن چیزی در چت است؟
    const teachCmd = this.parseNaturalTeachingCommand(userMessage);
    if (teachCmd) {
      if (teachCmd.type === 'qa') {
        const metric = this.teachInstantQA(teachCmd.question, teachCmd.answer, 16);
        return {
          isLearnedEvent: true,
          learnedData: teachCmd,
          reply: `✨ **در حافظه سیناپسی ثبت شد!**\n\nمن الان **۱۶ گام گرادیان کاهشی (Backpropagation)** روی شبکه عصبیم اجرا کردم و این الگو رو یاد گرفتم:\n- **ورودی/محرک:** «${teachCmd.question}»\n- **پاسخ شبکه:** «${teachCmd.answer}»\n\nالان همین عبارت رو ازم بپرس تا ببینی چطور بهت جواب میدم!`,
          confidence: 99,
          matchedSource: 'آموزش مستقیم در گفتگو (Online Continual Learning)',
          thinkingSteps: [
            `تشخیص دستور آموزش زنده در پیام کاربر («${teachCmd.question}»)`,
            `تبدیل عبارت به توکن‌های برداری و تزریق به لایه ورودی شبکه [10 → 14 → 10 → 8]`,
            `اجرای ۱۶ دور انتشار برگشتی خطا (Backpropagation)؛ کاهش Loss به ${metric.loss.toFixed(3)}`,
            `تثبیت سیناپس جدید در حافظه بلندمدت مدل`
          ],
          topPredictions: this.predictNextTokenDistribution(teachCmd.question, temperature).slice(0, 5)
        };
      } else if (teachCmd.type === 'fact') {
        this.customFacts.unshift(teachCmd.fact);
        const metric = this.teachInstantQA(teachCmd.fact.split(' ').slice(0, 3).join(' '), teachCmd.fact, 14);
        return {
          isLearnedEvent: true,
          learnedData: { question: teachCmd.fact.split(' ').slice(0, 3).join(' '), answer: teachCmd.fact },
          reply: `🧠 **دانش جدید جذب شد!**\n\nجمله **«${teachCmd.fact}»** به دیتاست شبکه عصبی من اضافه شد و وزن‌های سیناپسی با خطای **${metric.loss.toFixed(3)}** آپدیت شدند. از این به بعد توی تولید متن و پاسخ‌هام ازش استفاده می‌کنم!`,
          confidence: 98,
          matchedSource: 'تزریق دانش جدید (Continual Fact Learning)',
          thinkingSteps: [
            `استخراج گزاره دانشی جدید از پیام کاربر`,
            `توکنایز کردن جمله و افزودن به گراف واژگان (${this.vocab.length} توکن فعال)`,
            `به‌روزرسانی ماتریس توجه (Attention Logits) در ۱۴ گام گرادیان`,
            `آماده‌سازی مدل برای بازیابی این دانش در پرسش‌های بعدی`
          ],
          topPredictions: this.predictNextTokenDistribution(teachCmd.fact, temperature).slice(0, 5)
        };
      }
    }

    const topPredictions = this.predictNextTokenDistribution(cleanMsg, temperature).slice(0, 5);

    // ۲. اگر مغز کاملاً خام است (Epoch == 0)
    if (this.epoch === 0) {
      const rawOutput = this.generateText(userTokens.slice(0, 2).join(' '), 10, 1.5);
      return {
        reply: `🧪 **[خروجی مغز خام - Epoch 0]:**\n«${rawOutput}»\n\n---\n💡 *من هنوز آموزش ندیدم (وزن‌های شبکه عصبیم تصادفی هستند)!*\nمی‌تونی از نوار بالا دکمه **«⚡ آموزش مدل»** رو بزنی، یا همین‌جا توی چت بهم یاد بدی؛ مثلاً بنویس:\n\`یاد بگیر: وقتی گفتم ${userMessage} بگو سلام رفیق جان!\``,
        confidence: 14,
        matchedSource: 'شبکه عصبی خام (Epoch 0)',
        thinkingSteps: [
          `دریافت ${userTokens.length} توکن ورودی: [${userTokens.slice(0, 5).join('، ')}]`,
          `هشدار: تعداد دورهای آموزش Epoch = 0 است (وزن‌های سیناپسی در حالت تصادفی اولیه Xavier قرار دارند)`,
          `نمونه‌گیری تصادفی از توزیع یکنواخت واژگان (Loss فعلی: ${this.lastLoss.toFixed(2)})`
        ],
        topPredictions
      };
    }

    // ۳. جستجو در حافظه دیالوگ و آموزش‌های کاربر (Q&A Memory)
    let bestQA = null;
    let bestScore = 0;

    for (const qa of this.qaPairs) {
      let overlap = 0;
      for (const ut of userTokens) {
        if (ut.length < 2) continue;
        if (qa.qTokens.includes(ut)) {
          overlap += 1.6;
        } else if (qa.cleanQ.includes(ut)) {
          overlap += 0.9;
        }
      }
      if (cleanMsg === qa.cleanQ) overlap += 5.5;
      else if (cleanMsg.includes(qa.cleanQ) || qa.cleanQ.includes(cleanMsg)) overlap += 2.5;

      const normalized = overlap / Math.max(1, Math.sqrt(qa.qTokens.length));
      const userBoost = qa.isUserTaught ? 1.45 : 1.0;
      const weightedScore = normalized * userBoost * Math.min(2.2, 0.6 + (qa.learnedStrength || 1) * 0.2);

      if (weightedScore > bestScore) {
        bestScore = weightedScore;
        bestQA = qa;
      }
    }

    if (bestQA && bestScore >= 0.72) {
      const confidence = Math.min(99, Math.round(62 + bestScore * 16 + Math.min(18, this.epoch * 0.3)));
      return {
        reply: bestQA.answer,
        confidence,
        matchedSource: bestQA.isUserTaught
          ? `🎓 آموخته‌شده مستقیم از شما («${bestQA.question}»)`
          : `حافظه سیناپسی آموزش‌دیده (Epoch ${this.epoch})`,
        thinkingSteps: [
          `تجزیه پیام به ${userTokens.length} توکن و فعال‌سازی ${activeNeuronsCount} نورون در لایه خروجی`,
          `جستجو در ماتریس تداعی سیناپسی؛ تطابق بالا با الگوی «${bestQA.question}» (امتیاز: ${bestScore.toFixed(2)})`,
          `انتخاب پاسخ بهینه با اطمینان ${confidence}% و خطای شبکه ${this.lastLoss.toFixed(3)}`
        ],
        topPredictions
      };
    }

    // ۴. جستجو در جملات دیتاست یا تولید توکن‌به‌توکن (Autoregressive Generation)
    let seed = '';
    for (let i = userTokens.length - 1; i >= 0; i--) {
      const tok = userTokens[i];
      if (
        this.wordToIdx.has(tok) &&
        tok.length > 2 &&
        !['سلام', 'چطوری', 'چیه', 'چیست', 'آیا', 'چطور', 'بگو', 'میتونی'].includes(tok)
      ) {
        seed = tok;
        break;
      }
    }

    const generated = this.generateText(seed, 16, temperature);
    const confidence = Math.min(91, Math.max(32, Math.round(100 - this.lastLoss * 20)));

    return {
      reply: `${generated}\n\n*(💡 اگه دوست داری به این سوال جواب دقیق‌تری بدم، روی دکمه **«🎓 آموزش پاسخ دلخواه»** زیر همین پیام کلیک کن تا یادش بگیرم!)*`,
      confidence,
      matchedSource: seed
        ? `تولید زنجیره‌ای توکن‌به‌توکن حول کلیدواژه «${seed}»`
        : `تولید خودکار مدل زبانی (Temperature = ${temperature})`,
      thinkingSteps: [
        `تحلیل توکن‌های ورودی: [${userTokens.join('، ')}]`,
        seed
          ? `یافتن لنگر معنایی «${seed}» در گراف واژگان و محاسبه احتمالات توکن بعدی (Softmax)`
          : `عدم وجود پاسخ مستقیم در حافظه دیالوگ؛ سوئیچ به تولید زنجیره‌ای توکن‌به‌توکن (Autoregressive)`,
        `تولید دنباله کلمات با دمای خلاقیت ${temperature} و خطای شبکه ${this.lastLoss.toFixed(3)}`
      ],
      topPredictions
    };
  }

  teachInstantQA(question, answer, epochs = 14) {
    this.addQAPair(question, answer, true);
    const line = `سوال: ${question} | جواب: ${answer}`;
    this.prepareCorpus(line, false);
    let metric = { epoch: this.epoch, loss: this.lastLoss, accuracy: this.lastAccuracy };
    for (let i = 0; i < epochs; i++) {
      metric = this.trainOneEpoch(0.42);
    }
    return metric;
  }

  getStats() {
    return {
      epoch: this.epoch,
      vocabSize: Math.max(0, this.vocab.length - 3),
      contextCount: this.contextLogits.size,
      qaCount: this.qaPairs.length,
      trainingSamples: this.trainingPairs.length,
      loss: this.lastLoss,
      perplexity: Math.exp(Math.min(6, this.lastLoss)),
      accuracy: this.lastAccuracy
    };
  }
}

window.LLM_PRESET_CORPUSES = LLM_PRESET_CORPUSES;
window.MiniLLM = MiniLLM;
