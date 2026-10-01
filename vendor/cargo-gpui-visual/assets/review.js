"use strict";

// Timeline retains every virtual tick, including held images. Playback is review only.
for (const sequence of document.querySelectorAll(".sequence")) {
  const ticks = [...sequence.querySelectorAll("[data-tick]")];
  const slider = sequence.querySelector(".seq-slider");
  const play = sequence.querySelector(".seq-play");
  let current = 0;
  let timer = null;
  const show = (index) => {
    current = Math.max(0, Math.min(ticks.length - 1, index));
    slider.value = String(current);
    const tick = ticks[current];
    for (const button of ticks) button.classList.toggle("selected", button === tick);
    const viewer = sequence.querySelector(".viewer");
    const comparable = Boolean(tick.dataset.actual && tick.dataset.expected && tick.dataset.diff);
    viewer.dataset.comparable = String(comparable);
    for (const input of viewer.querySelectorAll("input[data-mode]")) {
      input.disabled = !comparable && input.dataset.mode !== "2up";
      if (!comparable) input.checked = input.dataset.mode === "2up";
    }
    for (const kind of ["actual", "expected", "diff"]) {
      for (const image of viewer.querySelectorAll(`[data-image="${kind}"]`)) {
        image.hidden = !tick.dataset[kind];
        if (tick.dataset[kind]) image.setAttribute("src", tick.dataset[kind]);
        else image.removeAttribute("src");
      }
      for (const link of viewer.querySelectorAll(`[data-link="${kind}"]`)) {
        link.hidden = !tick.dataset[kind];
        if (tick.dataset[kind]) link.setAttribute("href", tick.dataset[kind]);
        else link.removeAttribute("href");
      }
    }
    for (const [kind, other] of [["expected", "actual"], ["actual", "expected"]]) {
      viewer.querySelector(`.missing-${kind}`).hidden = Boolean(tick.dataset[kind]);
      const placeholder = viewer.querySelector(`[data-placeholder="${kind}"]`);
      if (tick.dataset[other]) placeholder.setAttribute("src", tick.dataset[other]);
      else placeholder.removeAttribute("src");
    }
    sequence.querySelector(".seq-caption").textContent = tick.dataset.caption;
    sequence.querySelector(".seq-state").textContent = tick.dataset.state;
    document.dispatchEvent(new Event("comparison-availability"));
  };
  const stop = () => { clearInterval(timer); timer = null; play.textContent = "Abspielen"; };
  const seek = (index) => { stop(); show(index); };
  slider.addEventListener("input", () => seek(Number(slider.value)));
  sequence.querySelector(".seq-prev").addEventListener("click", () => seek(current - 1));
  sequence.querySelector(".seq-next").addEventListener("click", () => seek(current + 1));
  sequence.querySelector(".seq-first").addEventListener("click", (e) => seek(Number(e.currentTarget.dataset.first)));
  for (const tick of ticks) tick.addEventListener("click", () => seek(Number(tick.dataset.tick)));
  sequence.querySelector(".sequence-filmstrip").addEventListener("toggle", (event) => {
    if (!event.currentTarget.open) return;
    for (const image of sequence.querySelectorAll("img[data-src]")) {
      image.src = image.dataset.src;
      image.removeAttribute("data-src");
    }
  });
  play.addEventListener("click", () => {
    if (timer !== null) { stop(); return; }
    if (current === ticks.length - 1) show(0);
    play.textContent = "Pause";
    timer = setInterval(() => {
      if (current === ticks.length - 1) stop(); else show(current + 1);
    }, 1000 / Number(sequence.dataset.hz));
  });
  show(0);
}
// Drag interactions for the visual review report. Embedded inline by
// report.rs and allowed by its exact sha256 in the CSP; nothing else may
// run. Without this script the swipe divider and the blend stay at 50 %.

for (const stage of document.querySelectorAll(".v-swipe .stage")) {
  const handle = stage.querySelector(".handle");
  let ratio = 0.5;
  const set = (value) => {
    ratio = Math.min(1, Math.max(0, value));
    stage.style.setProperty("--p", ratio.toFixed(4));
    handle.setAttribute("aria-valuenow", String(Math.round(ratio * 100)));
  };
  const follow = (event) => {
    const box = stage.getBoundingClientRect();
    if (box.width > 0) set((event.clientX - box.left) / box.width);
  };
  stage.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    event.preventDefault();
    stage.setPointerCapture(event.pointerId);
    follow(event);
  });
  stage.addEventListener("pointermove", (event) => {
    if (stage.hasPointerCapture(event.pointerId)) follow(event);
  });
  handle.addEventListener("keydown", (event) => {
    const step = event.shiftKey ? 0.1 : 0.02;
    const next = { ArrowLeft: ratio - step, ArrowRight: ratio + step, Home: 0, End: 1 }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    set(next);
  });
}

for (const slider of document.querySelectorAll(".v-onion .blend")) {
  const view = slider.closest(".v-onion");
  const apply = () => view.style.setProperty("--o", String(Number(slider.value) / 100));
  slider.addEventListener("input", apply);
  apply();
}

// Clicking a height mark opens the 1:1 window on exactly that row range. The
// window is precomputed in the report (data-y / data-h); turning the crop off
// again restores the default window around all differences.
const defaultWindows = new Map();
for (const viewer of document.querySelectorAll(".viewer")) {
  defaultWindows.set(viewer, {
    frames: [...viewer.querySelectorAll(".frame")].map((frame) => [frame, frame.style.cssText]),
    hint: viewer.querySelector(".crop-hint")?.textContent ?? "",
  });
}
function resetWindow(viewer) {
  const saved = defaultWindows.get(viewer);
  if (!saved) return;
  for (const [frame, css] of saved.frames) frame.style.cssText = css;
  const hint = viewer.querySelector(".crop-hint");
  if (hint) hint.textContent = saved.hint;
}
document.addEventListener("click", (event) => {
  const mark = event.target instanceof Element
    ? event.target.closest(".marks i[data-y], .to-stage[data-y]")
    : null;
  if (!mark) return;
  event.preventDefault(); // In 2-up the mark sits inside the link to the full image.
  // Inspector buttons sit beside the stage; bands sit inside it.
  const viewer = mark.closest(".viewer") ?? mark.closest("section")?.querySelector(".viewer");
  const crop = viewer?.querySelector(".m-crop");
  if (!crop) return;
  for (const frame of viewer.querySelectorAll(".frame")) {
    frame.style.setProperty("--y", `-${mark.dataset.y}px`);
    frame.style.setProperty("--h", `${mark.dataset.h}px`);
    // Row boxes also know their changed columns.
    if (mark.dataset.x !== undefined) {
      frame.style.setProperty("--x", `-${mark.dataset.x}px`);
      frame.style.setProperty("--w", `${mark.dataset.w}px`);
    }
  }
  const hint = viewer.querySelector(".crop-hint");
  if (hint) hint.textContent = `1:1-Ausschnitt um ${mark.dataset.rows} (${mark.dataset.hint})`;
  if (!crop.checked) {
    crop.checked = true;
    crop.dispatchEvent(new Event("change", { bubbles: true }));
  }
  viewer.closest("section")?.scrollIntoView({ block: "start" });
});
document.addEventListener("change", (event) => {
  const input = event.target;
  if (input instanceof HTMLInputElement && input.classList.contains("m-crop") && !input.checked) {
    resetWindow(input.closest(".viewer"));
  }
});

// "Ansicht für alle": sets every comparison at once. It shows the shared value
// only while all comparisons agree and "gemischt" otherwise; the chosen
// preference is remembered for later reports.
const globalMode = document.querySelector(".global-mode");
if (globalMode) {
  const STORAGE_KEY = "gpui-mobile-kit.visual-review.view";
  const allModes = [...globalMode.querySelectorAll("input[data-all-mode]")];
  const cropAll = globalMode.querySelector("#crop-all");
  const caseModes = () => [...document.querySelectorAll(".viewer input[data-mode]")];
  const caseCrops = () => [...document.querySelectorAll(".viewer .m-crop")];

  const sync = () => {
    const available = document.querySelector('.viewer[data-comparable="true"]') !== null;
    for (const input of allModes) input.disabled = !available && input.dataset.allMode !== "2up";
    const chosen = caseModes().filter((input) => input.checked && input.closest('.viewer[data-comparable="true"]')).map((input) => input.dataset.mode);
    const shared = chosen.length > 0 && chosen.every((mode) => mode === chosen[0]) ? chosen[0] : null;
    for (const input of allModes) input.checked = input.dataset.allMode === shared;
    const crops = caseCrops();
    const on = crops.filter((input) => input.checked).length;
    cropAll.checked = crops.length > 0 && on === crops.length;
    cropAll.indeterminate = on > 0 && on < crops.length;
    globalMode.classList.toggle("is-mixed", (available && shared === null) || cropAll.indeterminate);
  };
  const remember = () => {
    const mode = allModes.find((input) => input.checked)?.dataset.allMode;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ mode, crop: cropAll.checked }));
    } catch {
      // Storage can be unavailable for local files; the switch still works.
    }
  };
  const applyMode = (mode) => {
    for (const input of caseModes()) {
      const chosen = input.closest('.viewer[data-comparable="true"]') ? mode : "2up";
      if (input.dataset.mode === chosen) input.checked = true;
    }
  };
  const applyCrop = (on) => {
    for (const input of caseCrops()) {
      input.checked = on;
      if (!on) resetWindow(input.closest(".viewer"));
    }
  };

  for (const input of allModes) {
    input.addEventListener("change", () => {
      applyMode(input.dataset.allMode);
      sync();
      remember();
    });
  }
  cropAll.addEventListener("change", () => {
    applyCrop(cropAll.checked);
    sync();
    remember();
  });
  document.addEventListener("change", (event) => {
    if (event.target instanceof HTMLInputElement && event.target.closest(".viewer")) sync();
  });
  document.addEventListener("comparison-availability", sync);

  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null");
    if (stored && allModes.some((input) => input.dataset.allMode === stored.mode)) applyMode(stored.mode);
    if (stored && typeof stored.crop === "boolean") applyCrop(stored.crop);
  } catch {
    // Ignore unreadable preferences and keep the defaults.
  }
  globalMode.hidden = false;
  sync();
}

// Prüfstand shell: the entry in view, keyboard steps, table-of-contents filter,
// command copy and a local "seen" note. The report stays a complete, readable
// list without script; everything below is a shortcut, never an acceptance.
{
  const root = document.documentElement;
  const topbar = document.querySelector(".topbar");
  const controls = document.querySelector(".controls");
  const toc = document.querySelector(".toc");
  const reduced = matchMedia("(prefers-reduced-motion: reduce)");
  const measure = () => {
    for (const [element, name] of [[topbar, "--topbar"], [controls, "--toolbar"]]) {
      if (!element) continue;
      if (getComputedStyle(element).position === "sticky") {
        root.style.setProperty(name, `${element.offsetHeight}px`);
      } else {
        root.style.removeProperty(name);
      }
    }
  };
  measure();
  addEventListener("resize", measure);

  const entries = [...document.querySelectorAll("main section.case, main section.sequence, main .tile")];
  const idsOf = (entry) => (entry.id ? [entry.id] : [...entry.querySelectorAll(".shot[id]")].map((shot) => shot.id));
  const links = new Map();
  for (const link of document.querySelectorAll("nav a[href^='#']")) {
    links.set(decodeURIComponent(link.hash.slice(1)), link);
  }
  let active = null;
  const setActive = (entry) => {
    if (entry === active) return;
    for (const id of active ? idsOf(active) : []) links.get(id)?.removeAttribute("aria-current");
    active = entry;
    const link = entry && idsOf(entry).map((id) => links.get(id)).find(Boolean);
    for (const id of entry ? idsOf(entry) : []) links.get(id)?.setAttribute("aria-current", "true");
    if (link && toc) {
      const box = link.getBoundingClientRect();
      const frame = toc.getBoundingClientRect();
      if (box.top < frame.top + 48 || box.bottom > frame.bottom - 16) {
        toc.scrollTop += box.top - frame.top - frame.height / 3;
      }
    }
  };
  // "In view" is whatever crosses a thin line just below the sticky bars.
  const visible = new Set();
  const pick = () => setActive(entries.find((entry) => visible.has(entry)) ?? active);
  let observer = null;
  const observe = () => {
    observer?.disconnect();
    visible.clear();
    const sticky = (element) => element && getComputedStyle(element).position === "sticky" ? element.offsetHeight : 0;
    const line = sticky(topbar) + sticky(controls) + 24;
    const below = Math.max(0, innerHeight - line - 1);
    observer = new IntersectionObserver((changes) => {
      for (const change of changes) {
        if (change.isIntersecting) visible.add(change.target);
        else visible.delete(change.target);
      }
      pick();
    }, { rootMargin: `-${line}px 0px -${below}px 0px` });
    for (const entry of entries) observer.observe(entry);
  };
  if ("IntersectionObserver" in window) {
    observe();
    addEventListener("resize", observe);
  }

  const shown = (entry) => entry.offsetParent !== null;
  const go = (entry) => entry.scrollIntoView({ block: "start", behavior: reduced.matches ? "auto" : "smooth" });
  const step = (direction) => {
    const list = entries.filter(shown);
    if (list.length === 0) return;
    const index = active ? list.indexOf(active) : -1;
    const next = list[Math.max(0, Math.min(list.length - 1, index + direction))];
    if (next) go(next);
  };
  const stepper = document.querySelector(".topbar .step");
  if (stepper) {
    stepper.hidden = entries.length === 0;
    for (const button of stepper.querySelectorAll("button[data-step]")) {
      button.addEventListener("click", () => step(Number(button.dataset.step)));
    }
  }

  const nextRegion = (section) => {
    const regions = [...(section?.querySelectorAll("details.region") ?? [])];
    if (regions.length === 0) return;
    const open = regions.findIndex((region) => region.open);
    for (const region of regions) region.open = false;
    const next = regions[(open + 1) % regions.length];
    next.open = true;
    next.scrollIntoView({ block: "nearest" });
  };
  for (const button of document.querySelectorAll(".to-stage")) button.hidden = false;

  const copy = async (scope, button) => {
    const code = scope?.querySelector(".command code");
    if (!code) return;
    const text = code.textContent;
    let done = false;
    try {
      await navigator.clipboard.writeText(text);
      done = true;
    } catch {
      // file:// pages can lack clipboard access; select the text instead.
      const range = document.createRange();
      range.selectNodeContents(code);
      getSelection().removeAllRanges();
      getSelection().addRange(range);
    }
    const target = button ?? scope.querySelector("button.copy");
    if (!target) return;
    const label = target.firstChild.textContent;
    target.firstChild.textContent = done ? "Kopiert " : "Markiert, ⌘C drücken ";
    setTimeout(() => { target.firstChild.textContent = label; }, 1600);
  };
  for (const button of document.querySelectorAll("button.copy")) {
    button.hidden = false;
    button.addEventListener("click", () => copy(button.closest(".accept"), button));
  }

  const SEEN = `gpui-visual.seen:${location.pathname}:`;
  for (const box of document.querySelectorAll("input[data-seen]")) {
    const id = box.dataset.seen;
    const mark = () => {
      const section = box.closest("section");
      if (section) for (const key of idsOf(section)) links.get(key)?.classList.toggle("seen", box.checked);
    };
    try {
      box.checked = localStorage.getItem(SEEN + id) === "1";
    } catch {
      // Storage can be unavailable for local files; the checkbox still works.
    }
    mark();
    box.addEventListener("change", () => {
      try {
        if (box.checked) localStorage.setItem(SEEN + id, "1");
        else localStorage.removeItem(SEEN + id);
      } catch {
        // Keep the in-page state only.
      }
      mark();
    });
  }

  const filter = document.getElementById("toc-filter");
  if (filter) {
    filter.hidden = false;
    filter.addEventListener("input", () => {
      const query = filter.value.trim().toLowerCase();
      for (const link of links.values()) {
        link.hidden = query !== "" && !link.textContent.toLowerCase().includes(query);
      }
    });
    filter.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") return;
      filter.value = "";
      filter.dispatchEvent(new Event("input"));
      filter.blur();
    });
  }

  // Fragment targets inside a closed gallery pair open that pair in place.
  const reveal = () => {
    const target = location.hash ? document.getElementById(decodeURIComponent(location.hash.slice(1))) : null;
    const pair = target?.closest("details");
    if (pair && !pair.open) {
      pair.open = true;
      target.scrollIntoView({ block: "start" });
    }
  };
  addEventListener("hashchange", reveal);
  reveal();

  const MODES = ["2up", "swipe", "onion", "diff"];
  document.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target;
    if (target instanceof HTMLElement && (target.isContentEditable || target.tagName === "TEXTAREA"
      || (target instanceof HTMLInputElement && !["radio", "checkbox", "range"].includes(target.type)))) return;
    const key = event.key.toLowerCase();
    if (key === "j" || key === "k") step(key === "j" ? 1 : -1);
    else if (["1", "2", "3", "4"].includes(key)) {
      const input = document.getElementById(`all-${MODES[Number(key) - 1]}`);
      if (!input || input.disabled || input.closest("[hidden]")) return;
      input.click();
    } else if (key === "z") document.getElementById("crop-all")?.click();
    else if (key === "r") nextRegion(active);
    else if (key === "c") copy(active?.querySelector(".accept"));
    else if (key === "/" && filter) filter.focus();
    else return;
    event.preventDefault();
  });
}
