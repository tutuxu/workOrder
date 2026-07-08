const MODIFIER_KEYS = new Set(["Control", "Alt", "Shift", "Meta"]);

function normalizeMainKey(key: string, code: string): string | null {
  if (key === " " || code === "Space") return "Space";
  if (key === "Escape" || code === "Escape") return "Escape";
  if (key === "CapsLock" || code === "CapsLock") return "CapsLock";
  if (key === "Delete" || code === "Delete") return "Delete";
  if (key === "Backspace" || code === "Backspace") return "Backspace";
  if (key === "Enter" || code === "Enter") return "Enter";
  if (key === "Tab" || code === "Tab") return "Tab";
  if (key.startsWith("Arrow")) return key;

  if (key.length === 1) {
    return key.toUpperCase();
  }

  if (code.startsWith("Key")) {
    return code.slice(3);
  }

  if (code.startsWith("Digit")) {
    return code.slice(5);
  }

  if (MODIFIER_KEYS.has(key)) {
    return null;
  }

  return key.length > 1 ? key : key.toUpperCase();
}

export function eventToBinding(e: KeyboardEvent): string | null {
  const isCapsLock = e.key === "CapsLock" || e.code === "CapsLock";
  const mainKey = normalizeMainKey(e.key, e.code);
  if (!mainKey) return null;

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey && !isCapsLock) parts.push("Shift");
  parts.push(mainKey);
  return parts.join("+");
}

export function bindingsEqual(a: string, b: string): boolean {
  return normalizeBinding(a) === normalizeBinding(b);
}

export function normalizeBinding(binding: string): string {
  const parts = binding.split("+").map((part) => part.trim()).filter(Boolean);
  const modifiers: string[] = [];
  let mainKey = "";

  for (const part of parts) {
    const upper = part === "Cmd" || part === "Meta" ? "Ctrl" : part;
    if (upper === "Ctrl" || upper === "Alt" || upper === "Shift") {
      if (!modifiers.includes(upper)) modifiers.push(upper);
      continue;
    }
    mainKey = upper;
  }

  modifiers.sort((a, b) => {
    const order = ["Ctrl", "Alt", "Shift"];
    return order.indexOf(a) - order.indexOf(b);
  });

  return mainKey ? [...modifiers, mainKey].join("+") : modifiers.join("+");
}

export function formatBindingDisplay(binding: string | undefined): string {
  if (!binding) return "未设置";
  return binding;
}

export function isModifierOnlyEvent(e: KeyboardEvent): boolean {
  return MODIFIER_KEYS.has(e.key) || e.key === "CapsLock";
}
