import type { Ref } from "vue";

const INDENT = "  ";

export function isTextInputElement(
  el: EventTarget | null,
): el is HTMLInputElement | HTMLTextAreaElement {
  if (!(el instanceof HTMLElement)) return false;
  if (el instanceof HTMLTextAreaElement) return true;
  if (el instanceof HTMLInputElement) {
    const type = el.type.toLowerCase();
    return (
      type === "text" ||
      type === "search" ||
      type === "url" ||
      type === "email" ||
      type === "password" ||
      type === "tel" ||
      type === ""
    );
  }
  return false;
}

export function isCapsLockKey(e: KeyboardEvent): boolean {
  return e.key === "CapsLock" || e.code === "CapsLock";
}

export function resolveTextInput(el: EventTarget | null): HTMLInputElement | HTMLTextAreaElement | null {
  if (isTextInputElement(el)) return el;
  if (isTextInputElement(document.activeElement)) return document.activeElement;
  if (el instanceof HTMLElement) {
    const inner = el.querySelector("input, textarea");
    if (isTextInputElement(inner)) return inner;
  }
  return null;
}

export function insertTextIndent(valueRef: Ref<string>, e: KeyboardEvent): boolean {
  if (!isCapsLockKey(e)) return false;
  const el = resolveTextInput(e.target);
  if (!el) return false;
  e.preventDefault();
  e.stopPropagation();
  const start = el.selectionStart ?? 0;
  const end = el.selectionEnd ?? 0;
  const value = valueRef.value;

  if (start !== end && el instanceof HTMLTextAreaElement) {
    const before = value.slice(0, start);
    const selected = value.slice(start, end);
    const after = value.slice(end);
    const indented = selected
      .split("\n")
      .map((line) => INDENT + line)
      .join("\n");
    valueRef.value = before + indented + after;
    const cursor = start + indented.length;
    requestAnimationFrame(() => {
      el.selectionStart = cursor;
      el.selectionEnd = cursor;
    });
    return true;
  }

  valueRef.value = value.slice(0, start) + INDENT + value.slice(end);
  const cursor = start + INDENT.length;
  requestAnimationFrame(() => {
    el.selectionStart = cursor;
    el.selectionEnd = cursor;
  });
  return true;
}

const FOCUSABLE_SELECTOR =
  'input:not([disabled]):not([type="hidden"]), textarea:not([disabled]), button:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function focusNextElement(container: HTMLElement, reverse = false) {
  const elements = Array.from(
    container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR),
  ).filter((el) => el.offsetParent !== null && !el.closest(".n-radio-input"));

  if (elements.length === 0) return;

  const active = document.activeElement as HTMLElement | null;
  let index = active ? elements.indexOf(active) : -1;
  if (index < 0) {
    elements[0]?.focus();
    return;
  }

  index = reverse
    ? (index - 1 + elements.length) % elements.length
    : (index + 1) % elements.length;
  elements[index]?.focus();
}

export function cycleOption<T extends string>(
  current: T,
  options: readonly { value: T }[],
): T {
  const index = options.findIndex((opt) => opt.value === current);
  const next = index < 0 ? 0 : (index + 1) % options.length;
  return options[next]!.value;
}
