/**
 * Notchy - Clean & Interactive Dynamic Island Playground
 * Zero external dependencies. Pure mathematical springs and DOM animations.
 */

document.addEventListener("DOMContentLoaded", () => {
  // Elements
  const notchEl = document.getElementById("sim-notch");
  const screenContent = document.getElementById("sim-screen");
  const eyesContainer = document.getElementById("sim-eyes");
  const eyes = document.querySelectorAll(".sim-eye");
  const modeButtons = document.querySelectorAll("[data-sim-mode]");
  const dodgeTrigger = document.getElementById("sim-dodge-trigger");
  const petBtn = document.getElementById("pet-mascot-btn");
  const netDownEl = document.getElementById("sim-net-down");
  const netUpEl = document.getElementById("sim-net-up");
  const lyricLineEl = document.getElementById("sim-lyric-line");
  const langButtons = document.querySelectorAll("[data-lang-switch]");

  let currentMode = "eyes";
  let isTucked = false;
  let currentLang = "id";

  // Bilingual Synced Lyrics Library (Original Fictional Track: Arunika - Langit Kertas)
  const sampleLyricsMap = {
    id: [
      "Kita lipat malam jadi perahu ⛵",
      "Hanyut pelan di langit kertas ✨",
      "Lampu kota jadi bintang kecil 🌃",
      "Lirik sinkron otomatis tanpa buka tab peramban ⚡",
    ],
    en: [
      "We fold the night into a paper boat ⛵",
      "Drifting gently across a paper sky ✨",
      "City lights turn into little stars 🌃",
      "Work and study seamlessly with live synchronized lyrics 🎧",
    ],
  };
  let lyricIndex = 0;

  // Language management
  const setLanguage = (lang) => {
    currentLang = lang;
    document.body.classList.remove("lang-en", "lang-id");
    document.body.classList.add(`lang-${lang}`);
    document.documentElement.lang = lang;
    localStorage.setItem("notchy_lang", lang);

    // Update active button state
    langButtons.forEach((btn) => {
      if (btn.getAttribute("data-lang-switch") === lang) {
        btn.classList.add("text-white", "bg-white/[0.12]", "shadow-sm");
        btn.classList.remove("text-slate-400");
      } else {
        btn.classList.remove("text-white", "bg-white/[0.12]", "shadow-sm");
        btn.classList.add("text-slate-400");
      }
    });

    // Update current lyric text immediately
    if (lyricLineEl) {
      const lyrics = sampleLyricsMap[lang] || sampleLyricsMap.id;
      lyricLineEl.textContent = lyrics[lyricIndex % lyrics.length];
    }
  };

  const initLanguage = () => {
    const savedLang = localStorage.getItem("notchy_lang") || "id";
    setLanguage(savedLang);

    langButtons.forEach((btn) => {
      btn.addEventListener("click", () => {
        const lang = btn.getAttribute("data-lang-switch");
        setLanguage(lang);
      });
    });
  };

  initLanguage();

  // Mode definitions
  const setMode = (mode) => {
    currentMode = mode;
    isTucked = mode === "tucked";

    // Reset notch classes
    notchEl.className = "sim-notch";
    notchEl.classList.remove("tinted");

    // Hide all internal layers
    document.querySelectorAll(".sim-layer").forEach((layer) => {
      layer.classList.add("hidden");
    });

    // Update button states
    modeButtons.forEach((btn) => {
      if (btn.getAttribute("data-sim-mode") === mode) {
        btn.classList.add("active");
      } else {
        btn.classList.remove("active");
      }
    });

    switch (mode) {
      case "eyes":
        notchEl.classList.add("mode-eyes");
        document.getElementById("layer-eyes")?.classList.remove("hidden");
        break;

      case "music":
        notchEl.classList.add("mode-music", "tinted");
        notchEl.style.setProperty("--sim-tint", "244 114 182");
        document.getElementById("layer-music")?.classList.remove("hidden");
        break;

      case "lyrics":
        notchEl.classList.add("mode-lyrics", "tinted");
        notchEl.style.setProperty("--sim-tint", "129 140 248");
        document.getElementById("layer-lyrics")?.classList.remove("hidden");
        break;

      case "network":
        notchEl.classList.add("mode-network", "tinted");
        notchEl.style.setProperty("--sim-tint", "52 211 153");
        document.getElementById("layer-network")?.classList.remove("hidden");
        break;

      case "toast":
        notchEl.classList.add("mode-toast");
        document.getElementById("layer-toast")?.classList.remove("hidden");
        break;

      case "timer":
        notchEl.classList.add("mode-timer", "tinted");
        notchEl.style.setProperty("--sim-tint", "251 191 36");
        document.getElementById("layer-timer")?.classList.remove("hidden");
        break;

      case "charging":
        notchEl.classList.add("mode-charging", "tinted");
        notchEl.style.setProperty("--sim-tint", "52 211 153");
        document.getElementById("layer-charging")?.classList.remove("hidden");
        break;

      case "tucked":
        notchEl.classList.add("mode-tucked");
        document.getElementById("layer-tucked")?.classList.remove("hidden");
        break;

      default:
        notchEl.classList.add("mode-eyes");
        document.getElementById("layer-eyes")?.classList.remove("hidden");
    }
  };

  // Mode button events
  modeButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const mode = btn.getAttribute("data-sim-mode");
      setMode(mode);
    });
  });

  // Mascot Eye Theme Switcher
  const themeButtons = document.querySelectorAll("[data-sim-theme]");
  let currentTheme = "classic";

  const setTheme = (theme) => {
    currentTheme = theme;
    const allEyeContainers = document.querySelectorAll(".sim-eyes-container");
    allEyeContainers.forEach((container) => {
      container.classList.remove("theme-classic", "theme-anime", "theme-pixel", "theme-cat", "theme-cartoon");
      if (theme !== "classic") {
        container.classList.add(`theme-${theme}`);
      }
    });

    themeButtons.forEach((btn) => {
      if (btn.getAttribute("data-sim-theme") === theme) {
        btn.classList.add("active");
      } else {
        btn.classList.remove("active");
      }
    });
  };

  themeButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const theme = btn.getAttribute("data-sim-theme");
      setTheme(theme);
      if (currentMode !== "eyes" && currentMode !== "charging") {
        setMode("eyes");
      }
    });
  });

  // Mascot Greeting interaction
  const greetMascot = () => {
    if (!eyesContainer) return;
    eyesContainer.className = "sim-eyes-container hearts";
    setTimeout(() => {
      eyesContainer.className = "sim-eyes-container happy";
      setTimeout(() => {
        eyesContainer.className = "sim-eyes-container";
      }, 700);
    }, 600);
  };

  petBtn?.addEventListener("click", greetMascot);
  notchEl?.addEventListener("click", () => {
    if (currentMode === "eyes") {
      greetMascot();
    }
  });

  // Dodge simulator trigger button
  if (dodgeTrigger) {
    dodgeTrigger.addEventListener("mouseenter", () => {
      if (currentMode !== "tucked") {
        setMode("tucked");
      }
    });
    dodgeTrigger.addEventListener("mouseleave", () => {
      if (currentMode === "tucked") {
        setMode("eyes");
      }
    });
  }

  // Mascot Eye Tracking Logic (with Dual-Layer Parallax Pupil & Charging support)
  screenContent?.addEventListener("mousemove", (e) => {
    if (currentMode !== "eyes" && currentMode !== "charging") return;
    const rect = screenContent.getBoundingClientRect();
    const notchRect = notchEl.getBoundingClientRect();

    const notchCenterX = notchRect.left + notchRect.width / 2;
    const notchCenterY = notchRect.top + notchRect.height / 2;

    const dx = (e.clientX - notchCenterX) / (rect.width / 2);
    const dy = (e.clientY - notchCenterY) / (rect.height / 2);

    const clampedX = Math.max(-1, Math.min(1, dx));
    const clampedY = Math.max(-0.5, Math.min(1, dy));

    const allEyeContainers = document.querySelectorAll(".sim-eyes-container");
    allEyeContainers.forEach((container) => {
      if (!container.classList.contains("hearts") && !container.classList.contains("happy")) {
        container.style.transform = `translate(${clampedX * 10}px, ${clampedY * 5}px) rotate(${clampedX * 7}deg)`;
        container.style.setProperty("--sim-lx", (clampedX * 10) + "px");
        container.style.setProperty("--sim-ly", (clampedY * 6) + "px");
      }
    });
  });

  // Periodic blinking
  setInterval(() => {
    if (currentMode === "eyes" && !eyesContainer?.classList.contains("hearts")) {
      eyes.forEach((eye) => eye.classList.add("blink"));
      setTimeout(() => {
        eyes.forEach((eye) => eye.classList.remove("blink"));
      }, 140);
    }
  }, 3500);

  // Synced Lyrics Ticker Simulation
  setInterval(() => {
    if (lyricLineEl && currentMode === "lyrics") {
      const list = sampleLyricsMap[currentLang] || sampleLyricsMap.id;
      lyricIndex = (lyricIndex + 1) % list.length;
      lyricLineEl.style.opacity = "0";
      lyricLineEl.style.transform = "translateY(4px)";
      setTimeout(() => {
        lyricLineEl.textContent = list[lyricIndex];
        lyricLineEl.style.opacity = "1";
        lyricLineEl.style.transform = "translateY(0)";
      }, 200);
    }
  }, 3400);

  // Network Speed Randomizer
  setInterval(() => {
    if (netDownEl && netUpEl && currentMode === "network") {
      const downSpeed = (24.6 + (Math.random() * 10 - 5)).toFixed(1);
      const upSpeed = (5.8 + (Math.random() * 2 - 1)).toFixed(1);
      netDownEl.textContent = `${downSpeed} MB/s`;
      netUpEl.textContent = `${upSpeed} MB/s`;
    }
  }, 950);

  // ---------------------------------------------------------------------------
  // Dynamic GitHub Release Version Sync
  // ---------------------------------------------------------------------------
  let activeCleanVersion = "0.2.9";

  function applyReleaseData(releaseInfo) {
    if (!releaseInfo) return;
    const { tag, clean, installerUrl, portableUrl } = releaseInfo;
    if (clean) activeCleanVersion = clean;

    if (tag) {
      document.querySelectorAll(".data-version-text").forEach((el) => {
        el.textContent = tag;
      });
    }

    if (clean) {
      document.querySelectorAll(".data-version-clean").forEach((el) => {
        el.textContent = clean;
      });
    }

    if (installerUrl) {
      document.querySelectorAll("[data-download-installer]").forEach((el) => {
        el.href = installerUrl;
      });
    }

    if (portableUrl) {
      document.querySelectorAll("[data-download-portable]").forEach((el) => {
        el.href = portableUrl;
      });
    }
  }

  async function fetchLatestRelease() {
    const CACHE_KEY = "notchy_latest_release";
    const CACHE_TTL = 15 * 60 * 1000; // 15 minutes

    // 1. Try reading from localStorage first for instant render
    try {
      const cached = localStorage.getItem(CACHE_KEY);
      if (cached) {
        const parsed = JSON.parse(cached);
        if (parsed && parsed.timestamp && (Date.now() - parsed.timestamp < CACHE_TTL)) {
          applyReleaseData(parsed);
          return;
        }
      }
    } catch (e) {
      // Ignore localStorage errors (e.g. private browsing mode)
    }

    // 2. Fetch latest release from GitHub API
    try {
      const res = await fetch("https://api.github.com/repos/RamantyaDS/Notchy/releases/latest", {
        headers: { Accept: "application/vnd.github.v3+json" }
      });
      if (!res.ok) return;

      const data = await res.json();
      const rawTag = data.tag_name || "";
      if (!rawTag) return;

      const tag = rawTag.startsWith("v") ? rawTag : "v" + rawTag;
      const clean = tag.replace(/^v/, "");

      // Find assets
      const assets = Array.isArray(data.assets) ? data.assets : [];
      const installerAsset = assets.find((a) =>
        typeof a.name === "string" && (a.name.endsWith("-setup.exe") || (a.name.endsWith(".exe") && !a.name.endsWith(".sig")))
      );
      const portableAsset = assets.find((a) =>
        typeof a.name === "string" && (a.name.endsWith("-portable.zip") || (a.name.endsWith(".zip") && !a.name.endsWith(".sig")))
      );

      const installerUrl = installerAsset?.browser_download_url || data.html_url || "https://github.com/RamantyaDS/Notchy/releases/latest";
      const portableUrl = portableAsset?.browser_download_url || data.html_url || "https://github.com/RamantyaDS/Notchy/releases/latest";

      const releaseInfo = {
        timestamp: Date.now(),
        tag,
        clean,
        installerUrl,
        portableUrl
      };

      applyReleaseData(releaseInfo);

      try {
        localStorage.setItem(CACHE_KEY, JSON.stringify(releaseInfo));
      } catch (e) {
        // Ignore quota/storage errors
      }
    } catch (err) {
      // Graceful fallback to static HTML defaults on network failure/rate limiting
    }
  }

  fetchLatestRelease();



  // Smooth scroll
  document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
    anchor.addEventListener("click", function (e) {
      const targetId = this.getAttribute("href");
      if (targetId && targetId !== "#") {
        const targetElement = document.querySelector(targetId);
        if (targetElement) {
          e.preventDefault();
          targetElement.scrollIntoView({
            behavior: "smooth",
            block: "start",
          });
        }
      }
    });
  });
});
