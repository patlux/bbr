/*
 * BBR find-in-page strategy — executed inside the active WKWebView through
 * the public `evaluateJavaScript` bridge.
 *
 * Rationale: WKWebView's native find API (`findString:withConfiguration:
 * completionHandler:`) reports only a boolean (`WKFindResult.matchFound`);
 * the count/ordinal APIs (`findAllString:`, `WKFindConfiguration.findAllMatches`)
 * were removed from modern SDKs and exist only as private SPI. This script is
 * the single deterministic search strategy the adapter uses for count,
 * ordinal, and navigation, so the three can never drift apart: every call
 * recomputes the full non-overlapping match list from the live DOM and derives
 * the current ordinal from the actual selection.
 *
 * Determinism contract (documented in crates/bbr-web-macos/src/find_mapping.rs):
 * - Runs only in the top-level frame of the active web view; same-origin
 *   iframes are included recursively, cross-origin frames are skipped.
 * - Hidden text (display:none, visibility:hidden, hidden attribute,
 *   script/style/template/head content) is skipped.
 * - Matching is non-overlapping, leftmost-first.
 * - Case folding never changes character count (multi-char folds fall back to
 *   identity), so match positions map 1:1 onto the original text.
 * - Whole-word boundaries: the characters immediately before/after a match
 *   must not be letters, combining marks, digits, or underscores.
 * - Navigation is stateless: the current match is read from the live
 *   selection; stepping without a match under the caret selects the first
 *   (Next) or last (Previous) match.
 *
 * Non-destructive contract (proven by the real-app E2E fixtures):
 * - The strategy may READ the DOM and move the selection, but never
 *   persistently mutates page structure, element attributes, styles, or
 *   globals. It inserts no nodes, changes no attributes, and writes no
 *   style values.
 * - Highlights (CSS Custom Highlight registry) and the selection are
 *   transient visual state, cleared by the `clear` op on finish/
 *   invalidate and on zero matches; nothing survives a closed session.
 * - `scrollIntoView` only adjusts the viewport scroll; it is not a DOM
 *   mutation.
 * - The strategy executes in WKContentWorld::defaultClientWorld (BBR's own
 *   client world), so its `__bbr*` globals and MutationObserver live in a
 *   separate JS global object and are never observable by page scripts;
 *   the page-world isolation fixture asserts exactly that.
 *
 * Reply grammar (parsed by `parse_outcome` in Rust):
 *   OK:<count>:<ordinal|-1>:<mutated 0|1>   or   ERR:internal
 *
 * The `__BBR_*` tokens are replaced by the Rust adapter; the file must keep
 * exactly one occurrence of each token. The code is intentionally ES5-shaped
 * so the pure helpers can be exercised with JavaScriptCore's JXA harness.
 */

// --- Pure helpers (no DOM; JXA-testable) ------------------------------------

// Case folding that never changes character count so folded positions map 1:1
// back onto the original text. Multi-char folds (e.g. U+0130) are identity.
function __bbrFoldChar(ch) {
  var folded = ch.toLowerCase();
  return folded.length === 1 ? folded : ch;
}

function __bbrFold(s) {
  var out = "";
  for (var i = 0; i < s.length; i++) {
    out += __bbrFoldChar(s.charAt(i));
  }
  return out;
}

var __bbrWordRe = /[\p{L}\p{M}\p{N}_]/u;
function __bbrIsWordChar(ch) {
  return __bbrWordRe.test(ch);
}

// Non-overlapping leftmost scan over text runs `[{node, text}]`.
// Returns `[{node, start, end}]` with end-exclusive positions in run text.
function __bbrFindMatches(runs, query, caseSensitive, wholeWord) {
  var matches = [];
  var needle = caseSensitive ? query : __bbrFold(query);
  var needleLen = needle.length;
  if (needleLen === 0) {
    return matches;
  }
  for (var r = 0; r < runs.length; r++) {
    var text = runs[r].text;
    var hay = caseSensitive ? text : __bbrFold(text);
    var i = 0;
    while (i + needleLen <= hay.length) {
      var idx = hay.indexOf(needle, i);
      if (idx === -1) {
        break;
      }
      var ok = true;
      if (wholeWord) {
        var before = text.charAt(idx - 1);
        var after = text.charAt(idx + needleLen);
        ok = !(before !== "" && __bbrIsWordChar(before))
          && !(after !== "" && __bbrIsWordChar(after));
      }
      if (ok) {
        matches.push({ node: runs[r].node, start: idx, end: idx + needleLen });
        i = idx + needleLen; // non-overlapping: continue after the match
      } else {
        i = idx + 1;
      }
    }
  }
  return matches;
}

// Target index for a step. `currentIdx` -1 means no match under the caret
// (fallback: first for Next, last for Previous). At a no-wrap boundary the
// index is unchanged: a deterministic no-op. `count` is the match count.
function __bbrStepIndex(currentIdx, count, direction, wrapAround) {
  if (count === 0) {
    return -1;
  }
  if (currentIdx === -1) {
    return direction === 1 ? 0 : count - 1;
  }
  var next = currentIdx + direction;
  if (next < 0 || next >= count) {
    if (wrapAround) {
      return (next + count) % count;
    }
    return currentIdx; // no-wrap boundary: deterministic no-op
  }
  return next;
}

function __bbrContains(match, node, offset) {
  return match.node === node && match.start <= offset && offset < match.end;
}

function __bbrEncodeOk(count, ordinal, mutated) {
  var ord = ordinal === null ? -1 : ordinal;
  return "OK:" + count + ":" + ord + ":" + (mutated ? 1 : 0);
}

// --- DOM-dependent helpers (only executed inside a real page) ----------------

var __bbrSkipTags = {
  SCRIPT: 1, STYLE: 1, NOSCRIPT: 1, TEMPLATE: 1,
  HEAD: 1, TITLE: 1, META: 1, LINK: 1
};

function __bbrVisibleInTree(node) {
  for (var el = node; el; el = el.parentElement) {
    var tag = el.tagName;
    if (__bbrSkipTags[tag]) {
      return false;
    }
    if (el.hidden) {
      return false;
    }
    var cs = window.getComputedStyle(el);
    if (cs.display === "none" || cs.visibility === "hidden") {
      return false;
    }
  }
  return true;
}

// Visible text runs in document order (main tree first, then same-origin
// iframes recursively; cross-origin frames are skipped).
function __bbrCollectRuns() {
  var runs = [];
  function walk(root) {
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
    var n;
    while ((n = walker.nextNode())) {
      var value = n.nodeValue;
      if (!value || value.length === 0) {
        continue;
      }
      if (!__bbrVisibleInTree(n.parentElement)) {
        continue;
      }
      runs.push({ node: n, text: value });
    }
    var frames = root.querySelectorAll("iframe");
    for (var f = 0; f < frames.length; f++) {
      try {
        var doc = frames[f].contentDocument;
        if (doc && doc.body) {
          walk(doc.body);
        }
      } catch (e) {
        // Cross-origin frame: not accessible, skipped deterministically.
      }
    }
  }
  walk(document.body || document);
  return runs;
}

// Index of the match containing the selection start, or -1.
function __bbrSelectionMatch(matches) {
  var sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) {
    return -1;
  }
  var range = sel.getRangeAt(0);
  var node = range.startContainer;
  if (node.nodeType !== Node.TEXT_NODE) {
    return -1;
  }
  var offset = range.startOffset;
  for (var i = 0; i < matches.length; i++) {
    if (__bbrContains(matches[i], node, offset)) {
      return i;
    }
  }
  return -1;
}

function __bbrSelectMatch(match) {
  var range = document.createRange();
  range.setStart(match.node, match.start);
  range.setEnd(match.node, match.end);
  var sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
  if (match.node.parentElement) {
    match.node.parentElement.scrollIntoView({ block: "nearest" });
  }
}

function __bbrClearSelection() {
  var sel = window.getSelection();
  if (sel) {
    sel.removeAllRanges();
  }
}

// Highlight every match through the CSS Custom Highlight API (Safari 17.2+).
// Rendering is capped for pages with huge match sets; the reported count is
// never capped.
var __bbrHighlightName = "bbr-find";
var __bbrHighlightCap = 1000;

function __bbrHighlightAll(matches) {
  try {
    if (typeof CSS === "undefined" || !CSS.highlights) {
      return;
    }
    var cap = matches.length < __bbrHighlightCap ? matches.length : __bbrHighlightCap;
    var highlight = new Highlight();
    for (var i = 0; i < cap; i++) {
      var r = document.createRange();
      r.setStart(matches[i].node, matches[i].start);
      r.setEnd(matches[i].node, matches[i].end);
      highlight.add(r);
    }
    CSS.highlights.set(__bbrHighlightName, highlight);
  } catch (e) {
    // Unsupported or oversized: selection-only find.
  }
}

function __bbrClearHighlights() {
  try {
    if (typeof CSS !== "undefined" && CSS.highlights) {
      CSS.highlights.delete(__bbrHighlightName);
    }
  } catch (e) {
    // Nothing to clear.
  }
}

// Content mutations (child/character changes) between two strategy calls make
// the previous result stale. Attribute-only churn (animations, class toggles)
// is deliberately not observed.
function __bbrEnsureObserver() {
  if (window.__bbrFindObserver) {
    return;
  }
  window.__bbrFindMutated = false;
  try {
    var obs = new MutationObserver(function () {
      window.__bbrFindMutated = true;
    });
    obs.observe(document.documentElement, {
      subtree: true,
      childList: true,
      characterData: true
    });
    window.__bbrFindObserver = obs;
  } catch (e) {
    window.__bbrFindObserver = null;
  }
}

function __bbrTakeMutated() {
  var m = window.__bbrFindMutated === true;
  window.__bbrFindMutated = false;
  return m;
}

// --- Entry point -------------------------------------------------------------

function __bbrFindMain(config) {
  var mode = config.mode;
  try {
    if (mode === "clear") {
      __bbrClearHighlights();
      __bbrClearSelection();
      return __bbrEncodeOk(0, null, false);
    }

    var runs = __bbrCollectRuns();
    var matches = __bbrFindMatches(
      runs, config.query, config.caseSensitive, config.wholeWord
    );
    var count = matches.length;
    var mutated = __bbrTakeMutated();

    if (count === 0) {
      __bbrClearHighlights();
      __bbrClearSelection();
      return __bbrEncodeOk(0, null, mutated);
    }

    var currentIdx = __bbrSelectionMatch(matches);
    var targetIdx = currentIdx;

    if (mode === "start") {
      targetIdx = 0;
    } else if (mode === "step") {
      targetIdx = __bbrStepIndex(
        currentIdx, count, config.direction, config.wrapAround
      );
    }

    if (targetIdx !== currentIdx || mode === "start") {
      __bbrSelectMatch(matches[targetIdx]);
    }
    __bbrHighlightAll(matches);
    __bbrEnsureObserver();
    return __bbrEncodeOk(count, targetIdx + 1, mutated);
  } catch (e) {
    return "ERR:internal";
  }
}

// --- Adapter-filled configuration (tokens must each occur exactly once) ------

var __bbrConfig = {
  mode: "__BBR_MODE__",
  query: "__BBR_QUERY__",
  caseSensitive: "__BBR_CASE__",
  wholeWord: "__BBR_WORD__",
  wrapAround: "__BBR_WRAP__",
  direction: "__BBR_DIR__"
};

typeof document !== "undefined" ? __bbrFindMain(__bbrConfig) : "OK:0:-1:0";
