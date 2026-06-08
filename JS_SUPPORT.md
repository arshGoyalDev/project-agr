# JS Support in `project-agr`

This document outlines the JavaScript support provided by the `js-bindings` crate in the `project-agr` browser.

## Overview

The JS bindings provide a minimal DOM API for interaction between JavaScript and the Rust-based browser engine. The implementation is built using the Boa JavaScript engine and exposes a limited set of DOM functionalities.

## Runtime Setup

The JS runtime is initialized per tab and includes:

- A global `__rust__` object that provides native functions bound to Rust.
- A `console` object with a `log` method that forwards to Rust's stdout (supports multiple arguments).
- A `document` object with DOM selector methods, DOM creation methods, and the `body` / `documentElement` properties.
- A `Node` constructor and prototype methods for DOM interaction.
- An `Event` constructor and related event handling.

## Supported Features

### Console

- `console.log(...args)`: Logs one or more values to the standard output. All arguments are converted to strings and joined with spaces before being forwarded to Rust's stdout.

### Document

- `document.querySelector(selector)`: Returns the first `Node` matching the given CSS selector, or `null` if no match is found.
- `document.querySelectorAll(selector)`: Returns an array of `Node` objects matching the given CSS selector.
  - Supported selectors: tag names (e.g., `div`), class selectors (`.class`), ID selectors (`#id`), and descendant selectors (e.g., `div p`).
  - Unsupported selectors: universal (`*`), child (`>`), adjacent sibling (`+`), general sibling (`~`), attribute selectors (`[attr=value]`), pseudo-classes (`:hover`, `:focus`, etc.), and pseudo-elements (`::before`, `::after`).
- `document.getElementById(id)`: Returns the single `Node` with the specified ID, or `null` if it does not exist.
- `document.getElementsByClassName(className)`: Returns an array of `Node` objects that have the specified class name.
- `document.getElementsByTagName(tagName)`: Returns an array of `Node` objects that have the specified HTML tag name.
- `document.createElement(tagName)`: Creates and returns a new element `Node` with the given tag name. The node is registered in the handle map but not yet attached to the DOM.
- `document.createTextNode(text)`: Creates and returns a new text `Node` with the given string content. The node is registered in the handle map but not yet attached to the DOM.
- `document.body`: Getter that returns the `<body>` element as a `Node` (shorthand for `document.querySelector('body')`).
- `document.documentElement`: Getter that returns the `<html>` element as a `Node` (shorthand for `document.querySelector('html')`).

### Node Properties and Methods

- `node.children`: Returns an array of the node's direct child `Node` elements (ignores raw Text nodes).
- `node.getAttribute(attributeName)`: Returns the value of the specified attribute as a string, or `null` if the attribute does not exist.
- `node.innerHTML`:
  - Getter: Returns an HTML string representation of the node's children by walking the live DOM tree.
  - Setter: Replaces the node's children with the parsed HTML string. Triggers a relayout of the document.
- `node.textContent`:
  - Getter: Returns the concatenated text content of all descendant text nodes.
  - Setter: Replaces the node's children with a single text node containing the given string. Triggers a relayout.
- `node.appendChild(child)`: Appends `child` as the last child of the node. Sets the child's parent pointer and triggers a relayout.
- `node.insertBefore(newNode, referenceNode)`: Inserts `newNode` before `referenceNode` in the node's children list. If `referenceNode` is `null` or not found, `newNode` is appended. Triggers a relayout.
- `node.addEventListener(eventType, listener)`: Registers an event listener for the given event type on the node.
- `node.dispatchEvent(event)`: Dispatches an event to the node, triggering any registered listeners for that event type. Returns a boolean indicating if the default behavior should be prevented.

### Event

- `new Event(type)`: Creates a new event object with the given type.
- `event.preventDefault()`: Prevents the default action associated with the event (if any).

### Global Objects

- `__rust__`: Internal object providing the native bindings (not intended for direct use by page authors).
  - `__rust__.log(...args)`: Implements `console.log`; joins all arguments with spaces.
  - `__rust__.querySelector(selector)`: Implements `document.querySelector`.
  - `__rust__.querySelectorAll(selector)`: Implements `document.querySelectorAll`.
  - `__rust__.getElementById(id)`: Implements `document.getElementById`.
  - `__rust__.getElementsByClassName(className)`: Implements `document.getElementsByClassName`.
  - `__rust__.getElementsByTagName(tagName)`: Implements `document.getElementsByTagName`.
  - `__rust__.getAttribute(handle, attributeName)`: Implements `Node.prototype.getAttribute`.
  - `__rust__.innerHTML_set(handle, htmlString)`: Implements `Node.prototype.innerHTML` setter.
  - `__rust__.innerHTML_get(handle)`: Implements `Node.prototype.innerHTML` getter.
  - `__rust__.node_children(handle)`: Implements `Node.prototype.children` getter.
  - `__rust__.createElement(tagName)`: Implements `document.createElement`; returns a handle.
  - `__rust__.createTextNode(text)`: Implements `document.createTextNode`; returns a handle.
  - `__rust__.appendChild(parentHandle, childHandle)`: Implements `Node.prototype.appendChild`.
  - `__rust__.insertBefore(parentHandle, newHandle, refHandle)`: Implements `Node.prototype.insertBefore`.
  - `__rust__.textContent_get(handle)`: Implements `Node.prototype.textContent` getter.
  - `__rust__.textContent_set(handle, text)`: Implements `Node.prototype.textContent` setter.

## Script Loading

The HTML parser now handles `<script>` tags in a blocking manner, similar to a real browser:

- **Inline scripts**: Execution is paused as soon as the parser encounters an inline `<script>` block. The script is executed immediately, and parsing resumes afterward.
- **External scripts** (with `src`): The parser yields and waits for the script to be fetched. Once the resource is loaded, the script is executed and parsing resumes.
- **`defer` attribute**: External scripts with `defer` are collected and executed after the main HTML document has been fully parsed.
- **`async` attribute**: Treated similarly to `defer` in the current implementation — they are collected and run after parsing completes.

The HTML fetching pipeline (`loading::html_fetched`) was refactored to support this incremental/resumable parsing model.

## Limitations

- The JS engine does not support timers (`setTimeout`, `setInterval`).
- No support for XMLHttpRequest or fetch API.
- No CSSOM (e.g., `node.style` is not available).
- No layout information is exposed to JS (e.g., `offsetWidth`, `getBoundingClientRect`).
- Event objects lack properties beyond `type` and `do_default` (used internally for preventing default).
- The arrays/NodeLists returned by `querySelectorAll` and `getElementsBy*` methods are static and do not update dynamically when the DOM changes.
- Only a subset of DOM node types is supported (primarily Element and Text nodes).
- Event bubbling and capturing are not implemented; events are only dispatched to the target node.
- Error handling in JS is minimal; syntax errors are caught and logged to Rust's stderr, but exceptions are not propagated to JS try/catch.
- `node.parentNode`, `node.nextSibling`, `node.previousSibling`, and `node.nodeType` are not exposed.
- Newly created nodes via `createElement`/`createTextNode` must be explicitly appended to the DOM; they are not visible until attached.

## Implementation Notes

- The DOM state is managed via a thread-local `ACTIVE_DOM` variable that holds the current DOM tree and handle map during JS execution.
- Each DOM node is assigned a numeric handle when first accessed from JS, allowing the JS side to reference Rust-owned nodes.
- Nodes created via `createElement` / `createTextNode` are immediately registered in the handle map with a new handle but are not yet part of the tree.
- Setting `innerHTML`, `textContent`, calling `appendChild`, or `insertBefore` all set a `needs_relayout` flag; the actual relayout happens after the current JS execution block completes.
- The `innerHTML` getter reconstructs an HTML string by walking the live Rust DOM; it does not cache the result.
- The `textContent` getter recursively concatenates text from all descendant `Text` nodes.
- Event listeners are stored in a global `LISTENERS` map keyed by node handle and event type.

## Example Usage

```javascript
// Log multiple values
console.log("Count:", 42, "done");

// Select elements
let firstParagraph = document.querySelector("p");
let allParagraphs = document.querySelectorAll("p");
let mainContainer = document.getElementById("main");
let buttons = document.getElementsByClassName("btn");

// Access body and html root
let body = document.body;
let root = document.documentElement;

// Access children
if (mainContainer) {
  let childCount = mainContainer.children.length;
  console.log("Main container has", childCount, "children");
}

// Read innerHTML of the first paragraph
if (firstParagraph) {
  let html = firstParagraph.innerHTML;
  console.log("innerHTML:", html);
}

// Set innerHTML
if (firstParagraph) {
  firstParagraph.innerHTML = "<span>Updated text</span>";
}

// Read and set textContent
if (firstParagraph) {
  let text = firstParagraph.textContent;
  console.log("textContent:", text);
  firstParagraph.textContent = "Plain text replacement";
}

// Create and append a new element
let newDiv = document.createElement("div");
let textNode = document.createTextNode("Hello!");
newDiv.appendChild(textNode);
if (body) {
  body.appendChild(newDiv);
}

// Insert a node before an existing child
if (mainContainer && mainContainer.children.length > 0) {
  let banner = document.createElement("p");
  banner.textContent = "I was inserted first!";
  mainContainer.insertBefore(banner, mainContainer.children[0]);
}

// Add a click handler to a button
if (buttons.length > 0) {
  buttons[0].addEventListener("click", function () {
    console.log("Button clicked!");
  });
}

// Add a keydown handler to an input
let input = document.getElementsByTagName("input")[0];
if (input) {
  input.addEventListener("keydown", function () {
    let value = input.getAttribute("value");
    console.log(value);
  });
}

// Prevent default behaviour of an event
let form = document.querySelector("form");
if (form) {
  form.addEventListener("submit", function (e) {
    e.preventDefault();
  });
}

// Dispatch a custom event
let event = new Event("custom");
if (buttons.length > 0) {
  buttons[0].dispatchEvent(event);
}
```