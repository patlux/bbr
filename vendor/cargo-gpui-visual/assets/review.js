"use strict";
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
  const mark = event.target instanceof Element ? event.target.closest(".marks i[data-y]") : null;
  if (!mark) return;
  event.preventDefault(); // In 2-up the mark sits inside the link to the full image.
  const viewer = mark.closest(".viewer");
  const crop = viewer?.querySelector(".m-crop");
  if (!crop) return;
  for (const frame of viewer.querySelectorAll(".frame")) {
    frame.style.setProperty("--y", `-${mark.dataset.y}px`);
    frame.style.setProperty("--h", `${mark.dataset.h}px`);
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
    const chosen = caseModes().filter((input) => input.checked).map((input) => input.dataset.mode);
    const shared = chosen.length > 0 && chosen.every((mode) => mode === chosen[0]) ? chosen[0] : null;
    for (const input of allModes) input.checked = input.dataset.allMode === shared;
    const crops = caseCrops();
    const on = crops.filter((input) => input.checked).length;
    cropAll.checked = crops.length > 0 && on === crops.length;
    cropAll.indeterminate = on > 0 && on < crops.length;
    globalMode.classList.toggle("is-mixed", shared === null || cropAll.indeterminate);
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
    for (const input of caseModes()) if (input.dataset.mode === mode) input.checked = true;
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
