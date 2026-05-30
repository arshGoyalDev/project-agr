# JS Support in `project-agr`

This document outlines the JavaScript support provided by the `js-bindings` crate in the `project-agr` browser.

## Overview

The JS bindings provide a minimal DOM API for interaction between JavaScript and the Rust-based browser engine. The implementation is built using the Boa JavaScript engine and exposes a limited set of DOM functionalities.

## Runtime Setup

The JS runtime is initialized per tab and includes:

- A global `__rust__` object that provides native functions bound to Rust.
- A `console` object with a `log` method that forwards to Rust's stdout.
- A `document` object with `querySelectorAll` method.
- A `Node` constructor and prototype methods for DOM interaction.
- An `Event` constructor and related event handling.

## Supported Features

### Console

- `console.log(message)`: Logs a message to the standard output. The message is converted to a string via the Rust bindings.

### Document

- `document.querySelectorAll(selector)`: Returns a static NodeList of elements matching the given CSS selector.
  - Supported selectors: tag names (e.g., `div`), class selectors (`.class`), ID selectors (`#id`), and descendant selectors (e.g., `div p`).
  - Unsupported selectors: universal (`*`), child (`>`), adjacent sibling (`+`), general sibling (`~`), attribute selectors (`[attr=value]`), pseudo-classes (`:hover`, `:focus`, etc.), and pseudo-elements (`::before`, `::after`).

### Node Properties and Methods

- `node.getAttribute(attributeName)`: Returns the value of the specified attribute as a string, or `null` if the attribute does not exist.
- `node.innerHTML`:
  - Getter: Not implemented (returns empty string).
  - Setter: Replaces the node's children with the parsed HTML string. Setting innerHTML triggers a relayout of the document.
- `node.addEventListener(eventType, listener)`: Registers an event listener for the given event type on the node.
- `node.dispatchEvent(event)`: Dispatches an event to the node, triggering any registered listeners for that event type.

### Event

- `new Event(type)`: Creates a new event object with the given type.
- `event.preventDefault()`: Prevents the default action associated with the event (if any).

### Global Objects

- `__rust__`: Internal object providing the native bindings (not intended for direct use by page authors).
  - `__rust__.log(message)`: Implements `console.log`.
  - `__rust__.querySelectorAll(selector)`: Implements `document.querySelectorAll`.
  - `__rust__.getAttribute(handle, attributeName)`: Implements `Node.prototype.getAttribute`.
  - `__rust__.innerHTML_set(handle, htmlString)`: Implements `Node.prototype.innerHTML` setter.

## Limitations

- The JS engine does not support timers (`setTimeout`, `setInterval`).
- No support for XMLHttpRequest or fetch API.
- No CSSOM (e.g., `node.style` is not available).
- No layout information is exposed to JS (e.g., `offsetWidth`, `getBoundingClientRect`).
- Event objects lack properties beyond `type` and `do_default` (used internally for preventing default).
- The `NodeList` returned by `querySelectorAll` is static and does not update when the DOM changes.
- Only a subset of DOM node types is supported (primarily Element and Text nodes).
- Event bubbling and capturing are not implemented; events are only dispatched to the target node.
- Error handling in JS is minimal; syntax errors are caught and logged to Rust's stderr, but exceptions are not propagated to JS try/catch.

## Implementation Notes

- The DOM state is managed via a thread-local `ACTIVE_DOM` variable that holds the current DOM tree and handle map during JS execution.
- Each DOM node is assigned a numeric handle when first accessed from JS, allowing the JS side to reference Rust-owned nodes.
- Setting `innerHTML` marks the document as needing relayout, which occurs after the current JS execution block completes.
- Event listeners are stored in a global `LISTENERS` map keyed by node handle and event type.

## Example Usage

```javascript
// Log a message
console.log("Hello from JS");

// Select all paragraph elements
let paragraphs = document.querySelectorAll("p");

// Change the inner HTML of the first paragraph
if (paragraphs.length > 0) {
  paragraphs[0].innerHTML = "<span>Updated text</span>";
}

// Add a click handler to a button
let button = document.querySelectorAll(".button");
button.addEventListener("click", function () {
  console.log("Button clicked!");
});

// Add a keydown handler to and input
let input = document.querySelectorAll("input")[0];
input.addEventListener("keydown", function () {
  let value = input.getAttribute("value");
  console.log(value);
});

// Prevent default behaviour of an event
let form = document.querySelectorAll("form")[0];
form.addEventListener("submit", function (e) {
  e.preventDefault();
});

// Dispatch a custom event
let event = new Event("custom");
button.dispatchEvent(event);
```
