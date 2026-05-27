# CSS Support in `project-agr`

This document separates three things that are easy to confuse in the current codebase:

1. CSS syntax the parser accepts.
2. CSS declarations that are stored in the DOM style map.
3. CSS properties that actually affect layout or paint.

The third list is much smaller than the first two.

## Cascade Model

The cascade implementation is in `crates/css-parser/src/style.rs`.

- Inherited defaults:
  - `font-size: 16px`
  - `font-style: normal`
  - `font-weight: normal`
  - `font-family: sans-serif`
  - `color: black`
- Selector priority:
  - tag selector: `1`
  - class selector: `10`
  - id selector: `100`
  - descendant selector: sum of the nested selectors
- Inline styles use base priority `1000`.
- `!important` adds `10000`.
- Equal priority overwrites previous values because `>=` is used.

## Supported Selector Syntax

### Supported

- Tag selectors: `p`
- Class selectors: `.note`
- ID selectors: `#main`
- Descendant selectors: `article p`, `.card .title`, `#app form input`
- Comma-separated selector groups: `h1, h2, h3`

### Unsupported

- Universal selector: `*`
- Child combinator: `>`
- Adjacent sibling: `+`
- General sibling: `~`
- Attribute selectors: `[type=text]`
- Pseudo-classes: `:hover`, `:focus`, `:checked`, `:nth-child(...)`
- Pseudo-elements: `::before`, `::after`
- Namespace selectors

## Value Parsing

### General Behavior

- Property names are parsed with a restricted `word()` grammar.
- Property values are captured as raw strings until `;` or `}`.
- Unknown properties are preserved in the style map.
- CSS comments are not supported and can break parsing.

### `!important`

- Supported only as a raw suffix at the end of a property value.
- Example: `color: red !important`

## Properties That Affect Rendering

These properties have real downstream behavior today.

### `color`

- Used for text color.
- Used for checkbox/radio inner fill color.
- Inherited.

Supported value forms:

- Named colors:
  - `black`
  - `white`
  - `red`
  - `green`
  - `blue`
  - `lightblue`
  - `gray`
  - `grey`
  - `yellow`
  - `orange`
  - `purple`
  - `transparent` as a special no-color case
- Hex:
  - `#rgb`
  - `#rrggbb`

Unsupported:

- `rgb()`
- `rgba()`
- `hsl()`
- `hsla()`
- `#rgba`
- `#rrggbbaa`
- most named CSS colors

### `background-color`

- Used for element background fills in `BlockLayout::paint`.
- Used for input and checkbox/radio inner backgrounds.
- Also used to derive the canvas page background by scanning `<html>` and `<body>`.

Important limitation:

- Backgrounds render over coarse layout rectangles only. There is no padding, clipping, or stacking context.

### `font-size`

- Inherited.
- Parsed from `px`.
- `%` values are resolved relative to the parent if the parent has a `px` size.
- Layout converts pixel values to `iced` text size using a `0.75` multiplier.

Unsupported:

- `em`, `rem`, `pt`, `vw`, `calc()`, `clamp()`

### `font-weight`

- Only `bold` is recognized specially.
- Anything else effectively becomes normal.

### `font-style`

- `italic` and `oblique` map to italic.
- Everything else becomes normal.

### `font-family`

- Inherited.
- Mapped through a small fallback table:
  - `monospace`, `courier`, `consolas` -> monospace
  - `serif`, `times`, `times new roman`, `georgia` -> serif
  - `cursive`, `comic sans ms` -> cursive
  - `fantasy`, `impact` -> fantasy
  - everything else -> sans-serif

Important limitation:

- Comma-separated font stacks are not parsed semantically. A value like `Arial, sans-serif` is treated as one string.

### `width`

- Used by block layout as an explicit element width when the value is in `px`.
- Used by form controls to size inputs.

Limitations:

- No min/max width.
- No percentages.
- No box model.

### `height`

- Used by block layout as an explicit element height when the value is in `px`.
- Used by form controls to size inputs.

Limitations match `width`.

### `display`

- Partially used.
- Child elements with `display: block` influence whether the parent chooses block layout.
- The browser default stylesheet uses `display` to classify many tags.

Major limitation:

- The element's own `display` value does not suppress layout or paint. `display: none` is not honored as a true hidden state.

### `border-color`

- Used only for input, checkbox, radio, and button border painting.

### `border-width`

- Used only for input, checkbox, radio, and button border painting.
- Parsed as `px`.

## Shorthand Support

### `font`

Supported very loosely:

- `italic` and `oblique` -> `font-style`
- `bold` -> `font-weight`
- tokens containing `%` or `px` -> `font-size`
- every other token overwrites `font-family`

This means realistic shorthand like:

```css
font: italic bold 16px/1.4 "Fira Sans", sans-serif;
```

will not be interpreted correctly. Line height is ignored, quoted families are not parsed structurally, and the last unmatched token tends to win.

## Properties That Parse but Do Not Meaningfully Work

These may survive parsing and appear in the node style map, but the renderer/layout engine does not really implement them:

- `padding`
- `margin`
- `border`
- `border-radius`
- `text-decoration`
- `line-height`
- `text-align` except for hard-coded `<center>`
- `position`
- `top`, `left`, `right`, `bottom`
- `overflow`
- `opacity`
- `visibility`
- `z-index`
- `float`
- `clear`
- `flex`, `grid`, and related properties
- table layout properties

## Inline Style Support

Inline `style="..."` attributes are parsed by reusing `CSSParser::body()`.

Supported characteristics:

- multiple declarations separated by `;`
- `!important`
- same property behavior as stylesheet rules

Limitations:

- same parser limitations as regular CSS
- comments unsupported
- no robust quoted-value parsing rules

## Built-In Default Stylesheet

The root `browser.css` currently provides:

- block display for a set of common structural elements
- `display: none` declarations for `template`, `script`, `style`, `meta`, `link`, and `title`
- text styling for `a`, `i`, `b`, `small`, `big`, `code`
- `pre { background-color: grey; }`
- `html { background-color: white; }`

Important caveat:

- Because `display: none` is not fully implemented, those hide rules are only advisory today.

## CSS Edge Cases and Bugs

- Selector parsing depends on whitespace-separated words, so several valid CSS constructs are rejected.
- Comments like `/* ... */` are not skipped.
- Invalid rules recover by skipping to `}` or `;`, which is forgiving but imprecise.
- Property names accept only alphanumeric characters plus `#-.%_` through the parser's word routine, which is broader than needed in some places and still incomplete in others.
- Unknown values are stored as-is and fail later only when a downstream consumer tries to interpret them.

## Practical Support Summary

If you want CSS that works reliably in this project today, stay close to:

- tag/class/id/descendant selectors
- `color`
- `background-color`
- `font-size` in `px` or `%`
- `font-style`
- `font-weight`
- `font-family` with a single generic-like family name
- `width` and `height` in `px`
- `border-color` and `border-width` on form controls
- `display: block` for crude structure

Everything beyond that should be treated as unsupported or experimental.
