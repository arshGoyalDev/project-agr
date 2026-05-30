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
};

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

function Event(type) {
  this.type = type;
  this.do_default = true;
}

Event.prototype.preventDefault = function () {
  this.do_default = false;
};
