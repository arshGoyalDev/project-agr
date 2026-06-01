
globalThis.window = globalThis;

console = {
  log: function (x) {
    __rust__.log(x);
  },
};

function Node(handle) {
  this.handle = handle;
}

document = {
  querySelectorAll: function (x) {
    let handles = __rust__.querySelectorAll(x);
    return handles.map(function (h) {
      return new Node(h);
    });
  },

  querySelector: function (x) {
    let handle = __rust__.querySelector(x);
    return handle !== null ? new Node(handle) : null;
  },

  getElementById: function (id) {
    let handle = __rust__.getElementById(id);
    return handle !== null ? new Node(handle) : null;
  },

  getElementsByClassName: function (className) {
    let handles = __rust__.getElementsByClassName(className);
    return handles.map(function (h) {
      return new Node(h);
    });
  },

  getElementsByTagName: function (tagName) {
    let handles = __rust__.getElementsByTagName(tagName);
    return handles.map(function (h) {
      return new Node(h);
    });
  },
};

Object.defineProperty(document, "body", {
  get: function () {
    return document.querySelector("body");
  }
});

Object.defineProperty(document, "documentElement", {
  get: function () {
    return document.querySelector("html");
  }
});

Node.prototype.getAttribute = function (attribute) {
  return __rust__.getAttribute(this.handle, attribute);
};

let LISTENERS = {};

Node.prototype.addEventListener = function (type, listener) {
  if (!LISTENERS[this.handle]) LISTENERS[this.handle] = {};
  let dict = LISTENERS[this.handle];

  if (!dict[type]) dict[type] = [];

  let list = dict[type];
  list.push(listener);
};

Node.prototype.dispatchEvent = function (event) {
  let type = event.type;
  let handle = this.handle;
  let list = (LISTENERS[handle] && LISTENERS[handle][type]) || [];

  for (let i = 0; i < list.length; i++) {
    list[i].call(this, event);
  }

  return event.do_default;
};

Object.defineProperty(Node.prototype, "innerHTML", {
  set: function (s) {
    __rust__.innerHTML_set(this.handle, s.toString());
  },
});

Object.defineProperty(Node.prototype, "children", {
  get: function () {
    let handles = __rust__.node_children(this.handle);
    return handles.map(function (h) {
      return new Node(h);
    });
  },
});

function Event(type) {
  this.type = type;
  this.do_default = true;
}

Event.prototype.preventDefault = function () {
  this.do_default = false;
};
