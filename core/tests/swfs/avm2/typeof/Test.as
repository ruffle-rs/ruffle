package  {
public dynamic class Test {
}

}

trace("// typeof undefined");
trace(typeof undefined);

trace("// typeof null");
trace(typeof null);

var s = false;
trace("// typeof s(false)");
trace(typeof s);

trace("// typeof s(true)");
s = true;
trace(typeof s);

var a = 10.5;
trace("// typeof a(10.5)");
trace(typeof a);

var b = -10;
trace("// typeof b(-10)");
trace(typeof b);

var c = 15;
trace("// typeof c(15)");
trace(typeof c);

trace("// typeof {}");
trace(typeof {});

var d = "test";
trace("// typeof d('test')");
trace(typeof d);

function test() {}
trace("// typeof test (function)");
trace(typeof test);

var xml_obj = new XML();
trace("// typeof xml_obj (XML)");
trace(typeof xml_obj);

var xmllist_obj = new XMLList();
trace("// typeof listlist_obj (XMLList)");
trace(typeof xmllist_obj);

function Blah() {
    trace("Blah constructor");
}

Blah.prototype = new XML();
var blah = new Blah();
trace(typeof(blah));

var xml = new XML();
// trace(typeof(xml));

function t1() {
    trace("t1");
}

function t2() {
    trace("t2");
}

t1.prototype = new XML();
t2.prototype = t1;
// trace(typeof t1);
// trace(typeof t2);

trace(typeof(new t1()));
trace(typeof(new t2()));
