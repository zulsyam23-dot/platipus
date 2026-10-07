//! The base stylesheet every program is given.
//!
//! Elements compile to plain HTML tags, and a bare tag carries whatever the
//! browser decided: a grey system button, a control in the browser's own font,
//! a list with the platform's bullets, a table with no rules at all. None of
//! that reads as one program, and none of it is controllable from `.plt`.
//!
//! So the whole surface is described here, once, in terms of the tokens below.
//! A program restyles it by overriding the tokens in a `theme` block rather than
//! by rewriting these rules, which is what makes a theme apply to every element
//! instead of only the ones a program remembered to restyle.

/// The design tokens every rule below is written in terms of.
pub const TOKENS: &str = r#"
  --plt-font: system-ui, -apple-system, "Segoe UI", sans-serif;
  --plt-mono: ui-monospace, "Cascadia Code", Consolas, monospace;

  --plt-bg: #f6f6f8;
  --plt-surface: #ffffff;
  --plt-surface-2: #f0f0f4;
  --plt-border: #d9d9e0;
  --plt-text: #16161c;
  --plt-muted: #6a6a78;
  --plt-accent: #2f6feb;
  --plt-accent-text: #ffffff;
  --plt-danger: #c0392b;
  --plt-success: #1e7a48;

  --plt-radius: 8px;
  --plt-radius-sm: 5px;
  --plt-gap: 12px;
  --plt-pad: 12px;
  --plt-shadow: 0 1px 2px rgb(0 0 0 / 8%), 0 4px 12px rgb(0 0 0 / 6%);
  --plt-ring: 0 0 0 3px color-mix(in srgb, var(--plt-accent) 35%, transparent);
  --plt-code-bg: #f4f4f8;

  --plt-font-size: 15px;
  --plt-line-height: 1.5;
  --plt-control-gap: 6px;
  --plt-control-pad: 7px 14px;
  --plt-input-pad: 7px 10px;
  --plt-field-gap: 5px;
  --plt-check-size: 16px;
  --plt-cell-pad: 8px 12px;
  --plt-tab-pad: 8px 14px;
  --plt-item-pad: 6px 10px;
  --plt-menu-pad: 6px;
  --plt-overlay-pad: 10px 14px;
  --plt-bar-pad: 8px;
"#;

/// The same tokens under a dark `prefers-color-scheme`, so a program that
/// declares no theme still follows the reader's system setting.
pub const DARK_TOKENS: &str = r#"
  --plt-bg: #0d0d11;
  --plt-surface: #16161c;
  --plt-surface-2: #1f1f28;
  --plt-border: #2e2e3a;
  --plt-text: #e8e8ef;
  --plt-muted: #9a9aab;
  --plt-accent: #4d8dff;
  --plt-accent-text: #0d0d11;
  --plt-danger: #ff6b5e;
  --plt-success: #4ade80;
  --plt-shadow: 0 1px 2px rgb(0 0 0 / 40%), 0 4px 12px rgb(0 0 0 / 30%);
  --plt-code-bg: #101016;
"#;

/// Rules applied to everything, before any element rule. The form-control font
/// and colour are the load-bearing ones: a control that does not inherit them
/// renders in the platform's face at the platform's size, which is the single
/// most common reason a hand-written UI looks like it was assembled from parts
/// that came from different rooms.
pub const RESET: &str = r#"
*, *::before, *::after { box-sizing: border-box; }
html, body { height: 100%; }
body {
  margin: 0;
  background: var(--plt-bg);
  color: var(--plt-text);
  font-family: var(--plt-font);
  font-size: var(--plt-font-size);
  line-height: var(--plt-line-height);
  -webkit-font-smoothing: antialiased;
}
h1, h2, h3, h4, h5, h6, p, figure, blockquote, dl, dd { margin: 0; }
ul, ol { margin: 0; padding: 0; }
button, input, select, textarea {
  font: inherit;
  color: inherit;
  letter-spacing: inherit;
}
button { cursor: pointer; }
button:disabled, input:disabled, select:disabled, textarea:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
a { color: var(--plt-accent); }
:focus-visible { outline: none; box-shadow: var(--plt-ring); border-radius: var(--plt-radius-sm); }
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
}
"#;

/// One rule per builtin element, keyed by the class `class_of` generates.
pub const ELEMENTS: &[(&str, &str)] = &[
    // ---- layout ----------------------------------------------------------
    (
        "plt-page",
        "display: flex; flex-direction: column; min-height: 100%;",
    ),
    ("plt-container", "display: block; width: 100%;"),
    (
        "plt-column",
        "display: flex; flex-direction: column; gap: var(--plt-gap);",
    ),
    (
        "plt-row",
        "display: flex; flex-direction: row; align-items: center; gap: var(--plt-gap); flex-wrap: wrap;",
    ),
    (
        "plt-stack",
        "display: flex; flex-direction: column; gap: var(--plt-gap);",
    ),
    (
        "plt-scroll",
        "overflow: auto; overscroll-behavior: contain; padding: var(--plt-pad); background: var(--plt-surface); border: 1px solid var(--plt-border); border-radius: var(--plt-radius);",
    ),
    (
        "plt-card",
        "display: block; padding: var(--plt-pad); background: var(--plt-surface); border: 1px solid var(--plt-border); border-radius: var(--plt-radius);",
    ),
    (
        "plt-grid",
        "display: grid; gap: var(--plt-gap); grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));",
    ),
    (
        "plt-panel",
        "display: block; padding: var(--plt-pad); background: var(--plt-surface-2); border-radius: var(--plt-radius);",
    ),
    (
        "plt-splitter",
        "display: flex; gap: 1px; background: var(--plt-border); border: 1px solid var(--plt-border); border-radius: var(--plt-radius); overflow: hidden;",
    ),
    // The spacer exists to take the room between two things, so it has to grow.
    (
        "plt-spacer",
        "display: block; flex: 1 1 auto; min-width: 0; min-height: 0;",
    ),
    ("plt-viewport", "overflow: auto; height: 100%;"),
    (
        "plt-canvas",
        "display: block; max-width: 100%; border-radius: var(--plt-radius-sm); background: var(--plt-surface-2);",
    ),
    // ---- text ------------------------------------------------------------
    ("plt-text", "display: inline; overflow-wrap: anywhere;"),
    // Headings carry their own weight and rhythm; the tag alone leaves h1 through
    // h6 looking like six sizes of the same bold text.
    // The level is compiled into the tag, so these are keyed by tag rather than
    // by class: `Heading { level: 2 }` is an `h2` and nothing else.
    (
        ".plt-heading",
        "font-weight: 650; letter-spacing: -0.01em; color: var(--plt-text); margin: 0;",
    ),
    ("h1.plt-heading", "font-size: 1.6rem; line-height: 1.2;"),
    ("h2.plt-heading", "font-size: 1.3rem; line-height: 1.25;"),
    ("h3.plt-heading", "font-size: 1.1rem; line-height: 1.3;"),
    ("h4.plt-heading", "font-size: 1rem; line-height: 1.35;"),
    ("h5.plt-heading", "font-size: 0.9rem; line-height: 1.4;"),
    (
        "h6.plt-heading",
        "font-size: 0.82rem; line-height: 1.4; color: var(--plt-muted);",
    ),
    (
        "plt-link",
        "color: var(--plt-accent); text-decoration: underline; text-underline-offset: 2px; cursor: pointer;",
    ),
    (
        "plt-icon",
        "display: inline-flex; align-items: center; justify-content: center; font-style: normal; line-height: 1;",
    ),
    // ---- controls --------------------------------------------------------
    (
        "plt-button, .plt-command",
        "display: inline-flex; align-items: center; justify-content: center; gap: var(--plt-control-gap); padding: var(--plt-control-pad); border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); background: var(--plt-surface); color: var(--plt-text); font-weight: 500; line-height: 1.35; white-space: nowrap; transition: background 120ms ease, border-color 120ms ease;",
    ),
    (
        "plt-button:hover, .plt-command:hover",
        "background: var(--plt-surface-2); border-color: var(--plt-muted);",
    ),
    (
        "plt-button:active, .plt-command:active",
        "transform: translateY(1px);",
    ),
    (
        "plt-form",
        "display: flex; flex-direction: column; gap: var(--plt-gap);",
    ),
    // A field is a label above its control, which is what the markup already is,
    // so it only has to make that a grid rather than a stack of loose children.
    (
        "plt-field",
        "display: flex; flex-direction: column; gap: var(--plt-field-gap); min-width: 0;",
    ),
    (
        "input.plt-input, textarea.plt-textarea, select.plt-select",
        "width: 100%; padding: var(--plt-input-pad); border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); background: var(--plt-surface); color: var(--plt-text); min-width: 0;",
    ),
    (
        ".plt-row > input.plt-input, .plt-row > textarea.plt-textarea, .plt-row > select.plt-select",
        "width: auto; flex: 1 1 0;",
    ),
    (
        "input.plt-input::placeholder, textarea.plt-textarea::placeholder",
        "color: var(--plt-muted);",
    ),
    (
        "input.plt-input:hover, textarea.plt-textarea:hover, select.plt-select:hover",
        "border-color: var(--plt-muted);",
    ),
    (
        "textarea.plt-textarea, textarea.plt-code-editor",
        "min-height: 90px; resize: vertical; font-family: var(--plt-mono); font-size: 0.86rem; line-height: 1.55; tab-size: 2;",
    ),
    (
        "select.plt-select",
        "appearance: none; padding-right: 30px; background-image: linear-gradient(45deg, transparent 50%, currentColor 50%), linear-gradient(135deg, currentColor 50%, transparent 50%); background-position: right 13px center, right 8px center; background-size: 5px 5px, 5px 5px; background-repeat: no-repeat;",
    ),
    (
        "option.plt-option",
        "background: var(--plt-surface); color: var(--plt-text);",
    ),
    // One rule covers the checkbox and the switch because they are the same
    // control; the switch only adds its own shape on top.
    (
        "input.plt-checkbox, input.plt-radio, input.plt-switch",
        "width: var(--plt-check-size); height: var(--plt-check-size); padding: 0; margin: 0; accent-color: var(--plt-accent); cursor: pointer; flex: none;",
    ),
    ("input.plt-radio", "border-radius: 50%;"),
    (
        "input.plt-switch",
        "appearance: none; width: 34px; height: 20px; border-radius: 999px; background: var(--plt-border); position: relative; transition: background 140ms ease;",
    ),
    (
        "input.plt-switch::after",
        "content: \"\"; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: var(--plt-surface); transition: transform 140ms ease;",
    ),
    ("input.plt-switch:checked", "background: var(--plt-accent);"),
    (
        "input.plt-switch:checked::after",
        "transform: translateX(14px);",
    ),
    // A range input is drawn from scratch, because the platform track and thumb
    // cannot be coloured to match anything else on the page.
    (
        "input.plt-slider",
        "appearance: none; width: 100%; height: 20px; padding: 0; background: transparent; cursor: pointer;",
    ),
    (
        "input.plt-slider::-webkit-slider-runnable-track",
        "height: 4px; border-radius: 999px; background: var(--plt-border);",
    ),
    (
        "input.plt-slider::-moz-range-track",
        "height: 4px; border-radius: 999px; background: var(--plt-border);",
    ),
    (
        "input.plt-slider::-webkit-slider-thumb",
        "appearance: none; width: 14px; height: 14px; margin-top: -5px; border-radius: 50%; background: var(--plt-accent); border: 2px solid var(--plt-surface);",
    ),
    (
        "input.plt-slider::-moz-range-thumb",
        "width: 14px; height: 14px; border-radius: 50%; background: var(--plt-accent); border: 2px solid var(--plt-surface);",
    ),
    (
        "input.plt-color",
        "width: 44px; height: 30px; padding: 2px; cursor: pointer;",
    ),
    ("input.plt-file", "padding: 5px; cursor: pointer;"),
    (
        "input.plt-file::file-selector-button",
        "margin-right: 10px; padding: var(--plt-item-pad); border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); background: var(--plt-surface-2); color: var(--plt-text); font: inherit; cursor: pointer;",
    ),
    // ---- navigation ------------------------------------------------------
    (
        "plt-navigation, .plt-toolbar",
        "display: flex; align-items: center; gap: var(--plt-gap); padding: var(--plt-bar-pad) var(--plt-pad); background: var(--plt-surface); border-bottom: 1px solid var(--plt-border);",
    ),
    (
        "plt-sidebar",
        "padding: var(--plt-pad); background: var(--plt-surface); border-right: 1px solid var(--plt-border); flex: none;",
    ),
    (
        "plt-tabs",
        "display: flex; gap: 2px; border-bottom: 1px solid var(--plt-border);",
    ),
    (
        "plt-tab",
        "padding: var(--plt-tab-pad); border-bottom: 2px solid transparent; color: var(--plt-muted); cursor: pointer; user-select: none;",
    ),
    ("plt-tab:hover", "color: var(--plt-text);"),
    (
        "plt-tab[aria-selected=\"true\"], .plt-tab.plt-selected",
        "color: var(--plt-accent); border-bottom-color: var(--plt-accent);",
    ),
    (
        "plt-menu, .plt-tree",
        "display: flex; flex-direction: column; list-style: none;",
    ),
    (
        "plt-menu-item",
        "padding: var(--plt-item-pad); border-radius: var(--plt-radius-sm); cursor: pointer;",
    ),
    ("plt-menu-item:hover", "background: var(--plt-surface-2);"),
    (
        "plt-context-menu, .plt-popover",
        "padding: var(--plt-menu-pad); background: var(--plt-surface); border: 1px solid var(--plt-border); border-radius: var(--plt-radius); box-shadow: var(--plt-shadow); min-width: 160px;",
    ),
    // ---- surfaces and overlays -------------------------------------------
    (
        ".plt-sheet, .plt-toast, .plt-tooltip",
        "position: absolute; z-index: 20; padding: var(--plt-overlay-pad); background: var(--plt-surface); border: 1px solid var(--plt-border); border-radius: var(--plt-radius); box-shadow: var(--plt-shadow);",
    ),
    (
        ".plt-toast",
        "position: fixed; inset-block-end: 16px; inset-inline-end: 16px;",
    ),
    (
        ".plt-tooltip",
        "pointer-events: none; font-size: 0.85rem; color: var(--plt-muted);",
    ),
    (
        "dialog.plt-modal, dialog.plt-dialog",
        "padding: calc(var(--plt-pad) * 1.67); max-width: min(520px, 90vw); border: 1px solid var(--plt-border); border-radius: var(--plt-radius); background: var(--plt-surface); color: var(--plt-text); box-shadow: var(--plt-shadow);",
    ),
    ("dialog::backdrop", "background: rgb(0 0 0 / 45%);"),
    // ---- data ------------------------------------------------------------
    (
        "table.plt-table, table.plt-data-grid",
        "width: 100%; border-collapse: collapse; font-size: 0.94rem;",
    ),
    (
        ".plt-table th, .plt-data-grid th, .plt-table td, .plt-data-grid td",
        "padding: var(--plt-cell-pad); text-align: start; border-bottom: 1px solid var(--plt-border);",
    ),
    (
        ".plt-table th, .plt-data-grid th",
        "font-weight: 600; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.04em; color: var(--plt-muted); background: var(--plt-surface-2);",
    ),
    (
        ".plt-table tbody tr:hover, .plt-data-grid tbody tr:hover",
        "background: var(--plt-surface-2);",
    ),
    (
        ".plt-list, .plt-tree",
        "list-style: none; display: flex; flex-direction: column;",
    ),
    (
        ".plt-list > li, .plt-tree > li",
        "padding: var(--plt-item-pad); border-bottom: 1px solid var(--plt-border);",
    ),
    (
        ".plt-tree ul",
        "list-style: none; padding-inline-start: 18px; border-inline-start: 1px solid var(--plt-border);",
    ),
    // ---- editors ---------------------------------------------------------
    (
        ".plt-editor",
        "min-height: 90px; padding: var(--plt-input-pad); border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); background: var(--plt-surface); outline: none; overflow-wrap: anywhere;",
    ),
    (
        ".plt-editor:empty::before",
        "content: attr(data-placeholder); color: var(--plt-muted);",
    ),
    // A highlighted editor is three nodes: a shell the runtime builds, the
    // textarea the program bound to, and the `<pre>` that draws the highlight.
    // The overlay and the textarea have to occupy the same box, so the shell is
    // what gets the border and the padding and the two are layered inside it. The
    // textarea's own text has to be transparent, or it hides the highlight the
    // overlay exists to draw, and the overlay has to ignore the pointer, or the
    // caret cannot be placed in it.
    (
        ".plt-code-shell",
        "position: relative; display: block; border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); background: var(--plt-surface); overflow: hidden;",
    ),
    (
        ".plt-code-shell > textarea.plt-code-editor",
        "display: block; width: 100%; min-height: 140px; margin: 0; padding: var(--plt-input-pad); border: 0; background: transparent; color: transparent; caret-color: var(--plt-text); resize: vertical; outline: none; overflow: auto;",
    ),
    (
        ".plt-code-shell:focus-within",
        "border-color: var(--plt-accent); box-shadow: var(--plt-ring);",
    ),
    (
        ".plt-code-shell > pre.plt-code",
        "position: absolute; inset: 10px 12px; margin: 0; padding: 0; pointer-events: none; white-space: pre-wrap; overflow-wrap: break-word; color: var(--plt-text);",
    ),
    (
        ".plt-code .plt-tok",
        "color: var(--plt-accent); font-weight: 600;",
    ),
    // ---- feedback --------------------------------------------------------
    (
        "progress.plt-progress",
        "appearance: none; width: 100%; height: 8px; border: 0; border-radius: 999px; background: var(--plt-surface-2); overflow: hidden;",
    ),
    (
        "progress.plt-progress::-webkit-progress-bar",
        "background: var(--plt-surface-2); border-radius: 999px;",
    ),
    (
        "progress.plt-progress::-webkit-progress-value",
        "background: var(--plt-accent); border-radius: 999px;",
    ),
    (
        "progress.plt-progress::-moz-progress-bar",
        "background: var(--plt-accent); border-radius: 999px;",
    ),
    (
        "progress.plt-progress:indeterminate",
        "background: linear-gradient(90deg, transparent, var(--plt-accent), transparent); background-size: 40% 100%; animation: plt-slide 1.1s linear infinite;",
    ),
    (
        "progress.plt-progress:indeterminate::-webkit-progress-bar",
        "background: transparent;",
    ),
    (
        "progress.plt-progress:indeterminate::-webkit-progress-value",
        "background: transparent;",
    ),
    (
        ".plt-spinner",
        "width: 20px; height: 20px; border: 2px solid var(--plt-border); border-top-color: var(--plt-accent); border-radius: 50%; animation: plt-spin 700ms linear infinite;",
    ),
    (
        ".plt-loader",
        "border-radius: var(--plt-radius-sm); background: linear-gradient(90deg, var(--plt-surface-2) 25%, var(--plt-border) 50%, var(--plt-surface-2) 75%); background-size: 200% 100%; animation: plt-slide 1.3s ease-in-out infinite;",
    ),
    ("@keyframes plt-spin", "to { transform: rotate(360deg); }"),
    (
        "@keyframes plt-slide",
        "from { background-position: 200% 0; } to { background-position: -200% 0; }",
    ),
    // ---- media -----------------------------------------------------------
    (
        "img.plt-image",
        "display: block; max-width: 100%; height: auto; border-radius: var(--plt-radius-sm); background: var(--plt-surface-2);",
    ),
    (
        "video.plt-video",
        "display: block; max-width: 100%; border-radius: var(--plt-radius); background: #000;",
    ),
    ("audio.plt-audio", "width: 100%;"),
    // The three date/time controls are the same input element, so they share one
    // rule. They are named separately because the elements are: a program that
    // styles `Date` is not styling `Time`, and each has to answer to its own class
    // for that to mean anything.
    (
        "input.plt-date, input.plt-time",
        "font: inherit; color: var(--plt-text); background: var(--plt-surface); border: 1px solid var(--plt-border); border-radius: var(--plt-radius-sm); padding: var(--plt-item-pad); line-height: 1.35;",
    ),
    (
        "input.plt-date:hover, input.plt-time:hover",
        "border-color: var(--plt-muted);",
    ),
    (
        "input.plt-date:focus, input.plt-time:focus",
        "border-color: var(--plt-accent); outline: none; box-shadow: var(--plt-ring);",
    ),
    // A table is only readable once its cells line up and its head is set apart
    // from its body. Without these the browser's own table rules give it whatever
    // spacing it likes and no visual weight to the header row at all.
    (
        "tr.plt-table-row",
        "border-bottom: 1px solid var(--plt-border);",
    ),
    (
        "tr.plt-table-row:last-child",
        "border-bottom: 0; background: var(--plt-surface);",
    ),
    (
        "th.plt-header, td.plt-cell",
        "padding: var(--plt-cell-pad); text-align: left; border-bottom: 1px solid var(--plt-border);",
    ),
    (
        "th.plt-header",
        "font-weight: 600; color: var(--plt-muted); font-size: 0.85em; letter-spacing: 0.04em; text-transform: uppercase; background: var(--plt-surface-2); white-space: nowrap;",
    ),
    ("td.plt-cell", "color: var(--plt-text);"),
];

/// The wrapper the mount target is filled into. It is the only element that is
/// not a `.plt` element, so it is styled by name rather than by class.
pub const ROOT: &str = r#"
.plt-root {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  padding: calc(var(--plt-pad) * 1.67);
  gap: var(--plt-gap);
  background: var(--plt-bg);
  color: var(--plt-text);
}
"#;

/// The whole base stylesheet, in the order the layers have to arrive in:
/// tokens, then the reset, then the elements that override it.
pub fn stylesheet() -> String {
    let mut out = String::new();
    out.push_str(":root {\n");
    out.push_str(TOKENS);
    out.push_str("}\n");
    out.push_str("@media (prefers-color-scheme: dark) {\n  :root {\n");
    out.push_str(DARK_TOKENS);
    out.push_str("  }\n}\n");
    out.push_str(RESET);
    out.push_str(ROOT);
    for (selector, rule) in ELEMENTS {
        out.push_str(&format!("{} {{ {rule} }}\n", qualify(selector)));
    }
    out
}

/// A selector that names one of the element classes is written bare in the table
/// above, and needs the class dot here. Anything with other selector syntax in it
/// — a tag, a pseudo-class, an attribute, a keyframes rule — is already complete
/// and is left exactly as written.
///
/// Each comma-separated part is qualified on its own, because a rule that groups
/// `plt-button, .plt-command` only needs the dot on the part that is missing it,
/// and prefixing the whole string would produce a selector that matches nothing.
fn qualify(selector: &str) -> String {
    let mut out = String::with_capacity(selector.len());
    for (index, part) in selector.split(", ").enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        if part.starts_with("plt-") {
            out.push('.');
        }
        out.push_str(part);
    }
    out
}
