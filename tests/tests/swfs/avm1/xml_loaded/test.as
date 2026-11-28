// Reads and writes the `loaded` property of XML objects and of objects that
// only inherit from XML.prototype. XML objects keep the value themselves, behind
// a getter/setter that the first XML object adds to XML.prototype.

function own(o, name) {
    return o.hasOwnProperty(name);
}

function show(label, o) {
    trace(label + ": own " + own(o, "loaded") + ", typeof " + typeof(o.loaded) + ", value " + o.loaded);
}

function enumerates(o, name) {
    var found = false;
    for (var k in o) {
        if (k == name) {
            found = true;
        }
    }

    return found;
}

trace("// before any new XML");
show("XML.prototype", XML.prototype);
show("XMLNode.prototype", XMLNode.prototype);
show("Object.prototype", Object.prototype);
trace("XML.prototype status own " + own(XML.prototype, "status"));

var a = new XML();

trace("// after new XML");
show("a", a);
show("XML.prototype", XML.prototype);
show("XMLNode.prototype", XMLNode.prototype);
trace("XML.prototype status own " + own(XML.prototype, "status"));
trace("a.__proto__ == XML.prototype: " + (a.__proto__ == XML.prototype));
trace("XML.prototype enumerates loaded: " + enumerates(XML.prototype, "loaded"));
trace("a enumerates loaded: " + enumerates(a, "loaded"));

trace("// a.loaded = 5");
a.loaded = 5;
show("a", a);
show("XML.prototype", XML.prototype);
var b = new XML();
show("b (new XML)", b);

trace("// a.loaded = 0");
a.loaded = 0;
show("a", a);

trace("// a.loaded = 'false'");
a.loaded = "false";
show("a", a);

trace("// a.loaded = undefined");
a.loaded = undefined;
show("a", a);

trace("// a.loaded = null");
a.loaded = null;
show("a", a);

trace("// a.loaded = {}");
a.loaded = {};
show("a", a);

trace("// a.loaded = false, then undefined");
a.loaded = false;
a.loaded = undefined;
show("a", a);

trace("// a.loaded = true, then undefined");
a.loaded = true;
a.loaded = undefined;
show("a", a);

trace("// fresh XML, loaded = undefined");
var u = new XML();
u.loaded = undefined;
show("u", u);

trace("// a.loaded = NaN");
a.loaded = NaN;
show("a", a);

trace("// a.loaded = ''");
a.loaded = "";
show("a", a);

trace("// a.loaded = true");
a.loaded = true;
show("a", a);

trace("// delete a.loaded");
trace("returned " + delete a.loaded);
show("a", a);

trace("// XMLNode");
var n = new XMLNode(1, "n");
n.loaded = 5;
show("n", n);

trace("// object with XML.prototype as __proto__");
var o = {};
o.__proto__ = XML.prototype;
show("o", o);
o.loaded = 5;
show("o", o);

trace("// XML with custom __proto__");
var c = new XML();
var p = {};
p.__proto__ = XML.prototype;
c.__proto__ = p;
c.loaded = 5;
show("c", c);
show("p", p);

trace("// XML whose __proto__ is a plain object");
var d = new XML();
d.__proto__ = {};
d.loaded = 5;
show("d", d);
d.__proto__ = XML.prototype;
show("d (proto restored)", d);

trace("// subclass");
function MyXML() {
}
MyXML.prototype = new XML();
var m = new MyXML();
show("m", m);
show("MyXML.prototype", MyXML.prototype);
m.loaded = 5;
show("m", m);
show("MyXML.prototype", MyXML.prototype);

trace("// subclass calling super");
function MyXML2() {
    super();
}
MyXML2.prototype = new XML();
var m2 = new MyXML2();
m2.loaded = 5;
show("m2", m2);

trace("// own loaded via addProperty on XML");
var e = new XML();
e.addProperty("loaded", function() {
    trace("e getter");
    return "g";
}, function(v) {
    trace("e setter " + v);
});
show("e", e);
e.loaded = 5;

trace("// XML.prototype.loaded attributes");
ASSetPropFlags(XML.prototype, null, 0, 1);
trace("XML.prototype enumerates loaded after unhide: " + enumerates(XML.prototype, "loaded"));
var f = new XML();
f.loaded = true;
trace("returned " + delete XML.prototype.loaded);
show("XML.prototype", XML.prototype);
show("f", f);
var g = new XML();
show("g (new XML after delete)", g);
show("XML.prototype", XML.prototype);
g.loaded = 5;
show("g", g);

trace("Done");
