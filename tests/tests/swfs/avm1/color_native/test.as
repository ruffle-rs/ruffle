function enumerates(o, key) {
    for (var k in o) {
        if (k === key) return true;
    }
    return false;
}

trace("// new Color()");
var c = new Color();
trace(enumerates(c, "target"));
ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));

trace("// new Color(_root)");
var c = new Color(_root);
trace(enumerates(c, "target"));
ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));

trace("// new Date(), new Color()");
var c = new Date();

ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));

c.__initializeColor = function() {
    this.__proto__ = {
        __proto__: Color.prototype,
        __constructor__: Color
    };
    super();
};
c.__initializeColor();
trace(enumerates(c, "target"));
ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));

trace("// {}, new Color(_root)");
var c = {};

ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));
trace(c.target);

c.__initializeColor = function() {
    this.__proto__ = {
        __proto__: Color.prototype,
        __constructor__: Color
    };
    super(_root);
};
c.__initializeColor();

trace(enumerates(c, "target"));
ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));
trace(c.target);

trace("// new Color(), new Color(_root)");
var c = new Color();

ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));
trace(c.target);

c.__initializeColor = function() {
    this.__proto__ = {
        __proto__: Color.prototype,
        __constructor__: Color
    };
    super();
};
c.__initializeColor();

trace(enumerates(c, "target"));
ASSetPropFlags(c, null, 0, 65535);
trace(enumerates(c, "target"));
trace(c.target);

trace("");
trace("// Check how methods work on a non-native Color");

var o = {};
o.target = _root;
o.__proto__ = {
    __proto__: Color.prototype,
    __constructor__: Color
};

trace(o.getRGB());
trace(o.getTransform());

new Color(_root).setRGB(42);

trace(o.getRGB());

o.setRGB(24);
trace(new Color(_root).getRGB());
