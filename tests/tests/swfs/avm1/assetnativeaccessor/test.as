function enumerate(o) {
    var e = "  ";
    for (var p in o) {
        e += p + ",";
    }
    trace(e);
}

function testProps(props, obj) {
    var o;
    if (obj) {
        o = obj;
    } else {
        o = {};
    }
    trace("ASSetNativeAccessor(o, 200, \"" + props + "\")");
    ASSetNativeAccessor(o,200,props);
    enumerate(o);
    return o;
}


trace("ASSetNativeAccessor: " + ASSetNativeAccessor);
trace("typeof(ASSetNativeAccessor): " + typeof(ASSetNativeAccessor));
trace("typeof(ASSetNativeAccessor()): " + typeof(ASSetNativeAccessor));
var o = {};
trace("typeof(ASSetNativeAccessor(o)): " + typeof(ASSetNativeAccessor(o)));
trace("typeof(ASSetNativeAccessor(o, 2200)): " + typeof(ASSetNativeAccessor(o,2200)));
trace("typeof(ASSetNativeAccessor(o, 2200, \"a\")): " + typeof(ASSetNativeAccessor(o,2200,"a")));
trace("typeof(ASSetNativeAccessor(o, 2200, \"b\", undefined)): " + typeof(ASSetNativeAccessor(o,2200,"b",undefined)));

testProps("a");
testProps("a,b,c");
testProps("a,b,");
testProps(",a,b");
testProps(" a , b , c");
testProps("");
testProps(",");

var versions = testProps("1a,2b,3c,4d,5e,6f,7g,8h,9i,10j,11k,12l,13m");
trace("  a: " + versions.a);
trace("  b: " + versions.b);
trace("  c: " + versions.c);
trace("  d: " + versions.d);
trace("  e: " + versions.e);
trace("  f: " + versions.f);
trace("  g: " + versions.g);
trace("  h: " + versions.h);
trace("  i: " + versions.i);
trace("  j: " + versions.j);
trace("  k: " + versions.k);
trace("  l: " + versions.l);
trace("  m: " + versions.m);

var o = {};
o.a = "a";
o.b = "b";
o.c = "c";
o.d = "d";
o.e = "e";
o.f = "f";
o.g = "g";
o.h = "h";
o.i = "i";
o.j = "j";
o.k = "k";
o.l = "l";
o.m = "m";
var versions = testProps("1a,2b,3c,4d,5e,6f,7g,8h,9i,10j,11k,12l,13m", o);
trace("  a: " + versions.a);
trace("  b: " + versions.b);
trace("  c: " + versions.c);
trace("  d: " + versions.d);
trace("  e: " + versions.e);
trace("  f: " + versions.f);
trace("  g: " + versions.g);
trace("  h: " + versions.h);
trace("  i: " + versions.i);
trace("  j: " + versions.j);
trace("  k: " + versions.k);
trace("  l: " + versions.l);
trace("  m: " + versions.m);

var o_base = {};
o_base.a = "a_base";
o_base.b = "b_base";
o_base.c = "c_base";
o_base.d = "d_base";
o_base.e = "e_base";
o_base.f = "f_base";
o_base.g = "g_base";
o_base.h = "h_base";
o_base.i = "i_base";
o_base.j = "j_base";
o_base.k = "k_base";
o_base.l = "l_base";
o_base.m = "m_base";

var o = {};
o.__proto__ = o_base;

var versions = testProps("1a,2b,3c,4d,5e,6f,7g,8h,9i,10j,11k,12l,13m", o);
trace("  a: " + versions.a);
trace("  b: " + versions.b);
trace("  c: " + versions.c);
trace("  d: " + versions.d);
trace("  e: " + versions.e);
trace("  f: " + versions.f);
trace("  g: " + versions.g);
trace("  h: " + versions.h);
trace("  i: " + versions.i);
trace("  j: " + versions.j);
trace("  k: " + versions.k);
trace("  l: " + versions.l);
trace("  m: " + versions.m);

var o_base = {};
o_base.a = "a_base";
o_base.b = "b_base";
o_base.c = "c_base";
o_base.d = "d_base";
o_base.e = "e_base";
o_base.f = "f_base";
o_base.g = "g_base";
o_base.h = "h_base";
o_base.i = "i_base";
o_base.j = "j_base";
o_base.k = "k_base";
o_base.l = "l_base";
o_base.m = "m_base";

var o = {};
o.__proto__ = o_base;
o.a = "a";
o.b = "b";
o.c = "c";
o.d = "d";
o.e = "e";
o.f = "f";
o.g = "g";
o.h = "h";
o.i = "i";
o.j = "j";
o.k = "k";
o.l = "l";
o.m = "m";

var versions = testProps("1a,2b,3c,4d,5e,6f,7g,8h,9i,10j,11k,12l,13m", o);
trace("  a: " + versions.a);
trace("  b: " + versions.b);
trace("  c: " + versions.c);
trace("  d: " + versions.d);
trace("  e: " + versions.e);
trace("  f: " + versions.f);
trace("  g: " + versions.g);
trace("  h: " + versions.h);
trace("  i: " + versions.i);
trace("  j: " + versions.j);
trace("  k: " + versions.k);
trace("  l: " + versions.l);
trace("  m: " + versions.m);
