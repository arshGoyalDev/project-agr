# project-agr

`project-agr` is a small Rust browser experiment built as a workspace of focused crates:
It is rust implementation of the browser in the book [Web Browser Engineering](https://browser.engineering/index.html)

- `net`: URL parsing, HTTP(S)/file/data loading, redirects, gzip, and an in-memory cache.
- `html-parser`: a forgiving DOM builder with implicit `html`/`head`/`body` insertion.
- `css-parser`: selector parsing, cascade priority, inherited defaults, and inline style application.
- `layout`: block/inline layout, text measurement, input/button painting, display list generation, and view-source highlighting.
- `app`: the `iced` desktop shell, tab state, navigation, history, bookmarks, click handling, and canvas rendering.
- `js-bindings`: runtime and bindings for DOM events and handlers and basic js functions

The project is closer to a learning browser than a standards-compliant engine. It loads pages, builds a DOM, applies a small CSS subset, lays out text and simple form controls, and renders through a custom canvas display list.

## Workspace Layout

### `app`

- `main.rs`: starts the `iced` application and disables native window decorations.
- `browser.rs`: root browser state.
- `message.rs`: central message enum for UI, navigation, tabs, and typing.
- `net.rs`: async wrappers around the networking crate.
- `tab.rs`: per-tab document/history/focus state and form extraction.
- `canvas.rs`: canvas event handling and display-list painting.
- `window_controls.rs`: custom title-bar controls.
- `dom/helpers.rs`: title extraction, stylesheet discovery, inline style extraction, page background lookup.
- `update/*.rs`: behavior split by concern.
- `view/mod.rs`: the visible browser UI.

### `html-parser`

- `node.rs`: DOM node model.
- `parser.rs`: token-light HTML parser with implicit tag insertion and script/comment handling.

### `css-parser`

- `parser.rs`: CSS tokenization and rule parsing.
- `selector.rs`: tag, class, id, and descendant selectors.
- `style.rs`: cascade resolution and inherited property propagation.

### `layout`

- `document_layout.rs`: document root layout driver.
- `block_layout.rs`: the main layout engine.
- `line_layout.rs` / `text_layout.rs`: inline flow structures.
- `input_layout.rs`: rendering for text inputs, checkboxes, radios, and buttons.
- `display_list.rs`: abstract draw commands.
- `layout.rs`: constants and HTML entity decoding.
- `syntax_highlight.rs`: simple `view-source:` formatting.

### `net`

- `url_handler.rs`: URL parsing, HTTP(S)/file/data loading, redirects, gzip, and an in-memory cache

### `js-bindings`

- `runtime.rs`: initializes the dom state, and registers the js bindings
- `binding.rs`: rust function bindings for js functions

### Supporting Files

- `assets/browser.css`: built-in default stylesheet.
- `assets/bootstrap-icons.ttf`: icon font for the window chrome and browser controls.

## Architecture

The main runtime pipeline is:

1. `Browser::new` creates the first tab and dispatches `LoadUrl`.
2. `fetch_html_task` uses `net::URLHandler` to resolve and fetch the resource.
3. `loading::html_fetched` parses HTML into a DOM tree.
4. Stylesheet links and inline `<style>` blocks are collected.
5. `loading::css_fetched` loads `browser.css`, linked CSS, and inline CSS, then applies rules with `css_parser::style`.
6. `loading::js_fetched` loads linked js, then use `JsRuntime` to execute the scripts
6. `DocumentLayout::layout` builds layout boxes from the styled DOM.
7. `DocumentLayout::paint` emits a `DisplayList`.
8. `BrowserCanvas` renders the display list and handles scrolling, clicks, and typing events.

State is tab-local for DOM, layout, scroll offset, focus, title, and history. Network cache is process-global through a `lazy_static` `Mutex<HashMap<...>>`.

## Feature Inventory

### Navigation and Windowing

- Multiple tabs.
- Back/forward history per tab.
- Reload and hard reload (`Ctrl+R`, `Ctrl+Shift+R`).
- Address bar navigation.
- Heuristic search fallback to Google for non-URL input.
- Bookmarks and a generated `about:bookmarks` page.
- Custom window controls and draggable custom title bar.
- `about:blank`, `file:`, `data:`, `http:`, `https:`, and `view-source:` support.

### Networking

- Direct TCP HTTP client.
- TLS via `native-tls`.
- Redirect following with a hard cap of 10 redirects.
- `gzip` response decompression.
- `Transfer-Encoding: chunked` handling.
- `ETag` and `Last-Modified` conditional requests.
- Cache-Control `max-age` support.
- In-memory GET caching.

### HTML

- DOM tree with parent links.
- Implicit `html`, `head`, and `body` insertion.
- Self-closing tag list.
- HTML comment skipping.
- Raw-ish `<script>` text handling until `</script>`.
- Attribute capture and boolean attribute fallback.

### CSS

- Selectors: tag, `.class`, `#id`, descendant chains, comma-separated selector lists.
- Cascade priority: tag < class < id; inline styles override stylesheet rules; `!important` boosts priority.
- Inherited defaults for:
  - `font-size`
  - `font-style`
  - `font-weight`
  - `font-family`
  - `color`
- `%` font-size relative to parent `px`.
- `font` shorthand expansion into a very small subset.

### Layout and Rendering

- Block and inline layout split.
- Text measurement through `iced` advanced text primitives.
- Basic line wrapping.
- `<pre>` whitespace preservation.
- `<center>` and `<sup>` special-case behavior.
- Background fills for elements with `background-color`.
- Text color and simple font family/weight/style handling.
- Form control painting for:
  - text inputs
  - buttons
  - checkboxes
  - radio buttons
- Clickable links.
- Focused text input editing with blinking cursor.
- Form submission for GET and POST.
- `view-source:` page rendering through simple escaped `<pre>` output.

### JS

- `console.log` for logging to stdout.
- `document.querySelectorAll(selector)` returns a list of Node objects matching the CSS selector.
- `node.getAttribute(attribute)` returns the value of the attribute on the element.
- `node.innerHTML = htmlString` sets the inner HTML of the element (triggers relayout).
- `new Node(handle)` is a constructor used internally by the bindings to wrap a DOM handle.
- `document` is a global object with `querySelectorAll`.
- `Event` constructor and `event.preventDefault()`.
- `node.addEventListener(type, listener)` and `node.dispatchEvent(event)` for basic event handling.

## Current Behavior by Crate
### `net`

`URLHandler` strips fragments before fetching, parses the scheme manually, supports `view-source:` as a wrapper scheme, and resolves relative links against the current page. Requests are synchronous. Cache entries keep the body, timestamp, `ETag`, `Last-Modified`, and optional `max-age`.

### `html-parser`

The parser is a character scanner, not a tokenizer. It tracks three booleans: inside tag, inside comment, and inside script. Parsed nodes are stored as `Rc<RefCell<Node>>` with parent `Weak` references.

### `css-parser`

The parser stores declarations as string properties and only interprets a few of them later in layout. This is important: many CSS properties can be parsed and stored, but most do not affect layout or paint.

See [CSS_SUPPORT.md](/home/arshgoyal/Engineer/Developer/Rust/projects/project-agr/CSS_SUPPORT.md) for the CSS matrix in more detail.

### `layout`

`DocumentLayout` wraps a single `BlockLayout` rooted at the parsed DOM. `BlockLayout` decides block vs inline by looking at styled child display values and a few hard-coded tags (`input`, `button`). Painting outputs `Text`, `Rect`, and `Circle` commands.

### `app`

The app is an `iced` application with message routing split into:

- `update/window.rs`: input, clicks, typing, focus, cursor blinking, and resize.
- `update/navigation.rs`: history, bookmarks, reload, fragment scrolling, and loading.
- `update/loading.rs`: HTML/CSS parsing and final layout.
- `update/tabs.rs`: tab lifecycle.

### `js-bindings`

The JS runtime provides a minimal DOM API for interaction between JavaScript and the Rust engine.

See [JS_SUPPORT.md](/home/arshgoyal/Engineer/Developer/Rust/projects/project-agr/JS_SUPPORT.md) for the javascript support in more detail.
