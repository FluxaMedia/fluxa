const SEEK_STEP = 10;
const TEXT_ROLES = {
  text_input: ["text", "text"],
  search_input: ["search", "search"],
  password_input: ["password", "text"],
  url_input: ["url", "url"],
  number_input: ["text", "numeric"],
  multiline_input: ["text", "text"],
};
const STATIC_TAGS = { text: "p", heading: "h2" };

export function mountAccessibility(app, canvas, mirror) {
  const elements = new Map();
  let signature = "";
  let composing = false;
  let focusedId = null;
  let lastApp = null;
  let lastField = null;
  let sent = [];

  const send = (element) => {
    sent.push(element.value);
    if (sent.length > 64) sent.shift();
    app.set_focused_text(element.value);
  };

  const create = (node) => {
    const text = TEXT_ROLES[node.role];
    const tag = node.role === "multiline_input" ? "textarea" : text ? "input" : STATIC_TAGS[node.role] ?? "div";
    const element = document.createElement(tag);
    element.dataset.node = node.id;
    if (text) {
      if (tag === "input") element.type = text[0];
      element.inputMode = text[1];
      element.autocomplete = "off";
      element.autocapitalize = "off";
      element.spellcheck = false;
      if (node.role === "search_input") element.enterKeyHint = "search";
      element.addEventListener("focus", () => app.focus_node(node.id));
      element.addEventListener("compositionstart", () => (composing = true));
      element.addEventListener("compositionend", () => {
        composing = false;
        send(element);
      });
      element.addEventListener("input", () => send(element));
    } else if (STATIC_TAGS[node.role]) {
      element.tabIndex = -1;
    } else {
      element.tabIndex = -1;
      element.addEventListener("click", () => app.activate_node(node.id));
      if (node.role === "slider") {
        element.addEventListener("keydown", (event) => {
          const delta = { ArrowRight: SEEK_STEP, ArrowUp: SEEK_STEP, ArrowLeft: -SEEK_STEP, ArrowDown: -SEEK_STEP }[event.key];
          if (delta === undefined || !element._range) return;
          event.preventDefault();
          const [min, max, now] = element._range;
          app.media_command("seekTo", Math.min(Math.max(now + delta, min), max));
        });
      }
    }
    return element;
  };

  const apply = (element, node) => {
    const text = TEXT_ROLES[node.role];
    element.setAttribute("aria-label", node.label);
    const rect = element.style;
    rect.left = `${node.bounds[0]}px`;
    rect.top = `${node.bounds[1]}px`;
    rect.width = `${node.bounds[2]}px`;
    rect.height = `${node.bounds[3]}px`;
    if (text) return;
    if (STATIC_TAGS[node.role]) {
      element.removeAttribute("aria-label");
      element.textContent = node.label;
      return;
    }
    element.setAttribute("role", node.role === "tab" ? "tab" : node.role);
    element.textContent = node.value ? `${node.label}, ${node.value}` : node.label;
    if (node.checked !== null) element.setAttribute("aria-checked", String(node.checked));
    if (node.role === "tab") element.setAttribute("aria-selected", String(node.selected));
    if (node.range) {
      element._range = node.range;
      element.setAttribute("aria-valuemin", node.range[0]);
      element.setAttribute("aria-valuemax", node.range[1]);
      element.setAttribute("aria-valuenow", node.range[2]);
      if (node.value) element.setAttribute("aria-valuetext", node.value);
    }
  };

  const syncFocus = (tree) => {
    focusedId = tree.focused;
    const node = tree.nodes.find((candidate) => candidate.id === tree.focused);
    const element = node && TEXT_ROLES[node.role] ? elements.get(node.id) : null;
    if (!element) {
      if (mirror.contains(document.activeElement) && ["INPUT", "TEXTAREA"].includes(document.activeElement.tagName)) canvas.focus();
      return;
    }
    if (document.activeElement !== element) element.focus({ preventScroll: true });
    const text = app.focused_text() ?? "";
    if (node.id !== lastField) {
      lastField = node.id;
      lastApp = null;
      sent = [];
    }
    if (text !== lastApp) {
      lastApp = text;
      if (!composing && element.value !== text && !sent.includes(text)) element.value = text;
      if (element.value === text) sent = [];
    }
    if (!composing && tree.selection && document.activeElement === element) {
      const [start, end] = tree.selection;
      if (element.selectionStart !== start || element.selectionEnd !== end) element.setSelectionRange(start, end);
    }
  };

  const sync = () => {
    const json = app.accessibility_json();
    if (!json) return;
    if (json !== signature) {
      signature = json;
      const tree = JSON.parse(json);
      const seen = new Set();
      const ordered = [];
      for (const node of tree.nodes) {
        let element = elements.get(node.id);
        if (element && element.dataset.role !== node.role) {
          element.remove();
          element = null;
        }
        if (!element) {
          element = create(node);
          element.dataset.role = node.role;
          elements.set(node.id, element);
        }
        apply(element, node);
        seen.add(node.id);
        ordered.push(element);
      }
      for (const [id, element] of elements) {
        if (!seen.has(id)) {
          element.remove();
          elements.delete(id);
        }
      }
      ordered.forEach((element, index) => {
        if (mirror.children[index] !== element) mirror.insertBefore(element, mirror.children[index] ?? null);
      });
      syncFocus(tree);
    } else if (focusedId !== null && !composing) {
      syncFocus(JSON.parse(signature));
    }
  };

  canvas.addEventListener("pointerup", () => queueMicrotask(sync));
  return sync;
}
