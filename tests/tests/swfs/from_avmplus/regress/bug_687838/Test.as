/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}



import avmplus.*;
import avmshell.*;
import foo.*;
import com.adobe.test.Assert;

// var SECTION = "regress_687838";
// var VERSION = "AS3";
// var TITLE   = "Array prototype lookup";
// var bug = "687838";


// A+B+C
// * Array.prototype is modified.
// * Since B and C extend Array, their prototypes are array instances.
// * B's prototype is otherwise unaltered, C's prototype is directly modified.
// * B_sparse and C_sparse variants have prototypes that forced to be sparse.
dynamic class B extends Array { }
dynamic class B_sparse extends Array { }
dynamic class C extends Array { }
dynamic class C_sparse extends Array { }

// D+E+F
// * D and E's prototypes are replaced with fresh array instances.
//   F's prototype is left alone.
// * d_pre and e_pre are allocated before the replacement of the D and
//   E prototypes; d_post and e_post are allocated after the
//   replacement.
// * D's replacement prototype is otherwise unaltered, E replacement
//   and F's prototypes directly modified.

function D() { }
function E() { }
function F() { }

var a = new Array
// a_sparse is itself sparse
var a_sparse = new Array
a_sparse[2000] = "twothou on A";
var b = new B
var c = new C
var b_sparse = new B_sparse
var c_sparse = new C_sparse

// See notes with A+B+C above
Array   .prototype["n"]  = "named A";
Array   .prototype[1]    = "one A";
Array   .prototype[10]   = "ten A";
C       .prototype["n"]  = "named C";
C       .prototype[1]    = "one C";
C       .prototype[10]   = "ten C";
C_sparse.prototype["n"]  = "named C_sparse";
C_sparse.prototype[1]    = "one C_sparse";
C_sparse.prototype[10]   = "ten C_sparse";

// Encouraging B_sparse and C_sparse prototypes to be sparse without
// clobbering index 1000.
B_sparse.prototype[2000] = "twothou B_sparse";
C_sparse.prototype[2000] = "twothou C_sparse";

// See notes with E+D+F above

D       .prototype["n"]  = "named Dpre"
D       .prototype[1]    = "one Dpre"
D       .prototype[10]   = "ten Dpre"
E       .prototype["n"]  = "named Epre"
E       .prototype[1]    = "one Epre";
E       .prototype[10]   = "ten Epre";

const D_prototype_pre = D.prototype;
const E_prototype_pre = E.prototype;

// The _pre constructions deliberately occur before the replacement of
// the .prototype object in D and E.
var d_pre = new D
var e_pre = new E
var f = new F

D.prototype = new Array;
E.prototype = new Array;

// The _post constructions deliberately occur after the replacement of
// the .prototype object in D and E.
var d_post = new D
var e_post = new E

E       .prototype["n"]  = "named Epost";
E       .prototype[1]        = "one Epost";
E       .prototype[10]       = "ten Epost";
F       .prototype["n"]  = "named F";
F       .prototype[1]        = "one F";
F       .prototype[10]       = "ten F";

const ATC:Function = Assert.expectEq;

ATC('Arr .proto start1 array        named a', "named A",              a["n"]);
ATC('Cls .proto start1 array trans  named b', "named A",              b["n"]);
ATC('Cls .proto start1 array immed  named c', "named C",              c["n"]);
ATC('Fcn .proto object              named d', "named Dpre",       d_pre["n"]);
ATC('Fcn .proto start1 array trans  named d', "named A",         d_post["n"]);
ATC('Fcn .proto object              named e', "named Epre",       e_pre["n"]);
ATC('Fcn .proto start1 array immed  named e', "named Epost",     e_post["n"]);
ATC('Fcn .proto object              named f', "named F",              f["n"]);

ATC('Arr .proto start1 array       ref  1 a', "one A",                a[1]);
ATC('Arr .proto start1 self sparse ref  1 a', "one A",         a_sparse[1]);
ATC('Cls .proto start1 array trans ref  1 b', "one A",                b[1]);
ATC('Cls .proto sparse array trans ref  1 b', "one A",         b_sparse[1]);
ATC('Cls .proto start1 array immed ref  1 c', "one C",                c[1]);
ATC('Cls .proto sparse array immed ref  1 c', "one C_sparse",  c_sparse[1]);
ATC('Fcn .proto object             ref  1 d', "one Dpre",         d_pre[1]);
ATC('Fcn .proto start1 array trans ref  1 d', "one A",           d_post[1]);
ATC('Fcn .proto object             ref  1 e', "one Epre",         e_pre[1]);
ATC('Fcn .proto start1 array immed ref  1 e', "one Epost",       e_post[1]);
ATC('Fcn .proto object             ref  1 f', "one F",                f[1]);

ATC('Arr .proto start1 array       ref 10 a', "ten A",                a[10]);
ATC('Cls .proto start1 array trans ref 10 b', "ten A",                b[10]);
ATC('Cls .proto sparse array trans ref 10 b', "ten A",         b_sparse[10]);
ATC('Cls .proto start1 array immed ref 10 c', "ten C",                c[10]);
ATC('Cls .proto sparse array immed ref 10 c', "ten C_sparse",  c_sparse[10]);
ATC('Fcn .proto object             ref 10 d', "ten Dpre",         d_pre[10]);
ATC('Fcn .proto start1 array trans ref 10 d', "ten A",           d_post[10]);
ATC('Fcn .proto object             ref 10 e', "ten Epre",         e_pre[10]);
ATC('Fcn .proto start1 array immed ref 10 e', "ten Epost",       e_post[10]);
ATC('Fcn .proto object             ref 10 f', "ten F",                f[10]);

ATC('Arr .proto start1 array       ref 1k a', undefined,              a[1000]);
ATC('Cls .proto start1 array trans ref 1k b', undefined,              b[1000]);
ATC('Cls .proto start1 array immed ref 1k c', undefined,              c[1000]);
ATC('Cls .proto sparse array immed ref 1k c', undefined,       c_sparse[1000]);
ATC('Fcn .proto object             ref 1k d', undefined,          d_pre[1000]);
ATC('Fcn .proto object             ref 1k e', undefined,          e_pre[1000]);
ATC('Fcn .proto start1 array trans ref 1k d', undefined,         d_post[1000]);
ATC('Fcn .proto start1 array immed ref 1k e', undefined,         e_post[1000]);
ATC('Fcn .proto object             ref 1k f', undefined,              f[1000]);

Array   .prototype[0] = "zero A";
C       .prototype[0] = "zero C";
C_sparse.prototype[0] = "zero C_sparse";
E       .prototype[0] = "zero E";
F       .prototype[0] = "zero F";

ATC('Arr .proto start0 array        named a', "named A",               a["n"]);
ATC('Cls .proto start0 array trans  named b', "named A",               b["n"]);
ATC('Cls .proto start0 array immed  named c', "named C",               c["n"]);
ATC('Fcn .proto object              named d', "named Dpre",        d_pre["n"]);
ATC('Fcn .proto start0 array trans  named d', "named A",          d_post["n"]);
ATC('Fcn .proto object              named e', "named Epre",        e_pre["n"]);
ATC('Fcn .proto start0 array immed  named e', "named Epost",      e_post["n"]);
ATC('Fcn .proto object              named f', "named F",               f["n"]);

ATC('Arr .proto start0 array       ref  1 a', "one A",                 a[1]);
ATC('Cls .proto start0 array trans ref  1 b', "one A",                 b[1]);
ATC('Cls .proto start0 array immed ref  1 c', "one C",                 c[1]);
ATC('Fcn .proto object             ref  1 d', "one Dpre",          d_pre[1]);
ATC('Fcn .proto object             ref  1 e', "one Epre",          e_pre[1]);
ATC('Fcn .proto start0 array trans ref  1 d', "one A",            d_post[1]);
ATC('Fcn .proto start0 array immed ref  1 e', "one Epost",        e_post[1]);
ATC('Fcn .proto object             ref  1 f', "one F",                 f[1]);

ATC('Arr .proto start0 array       ref 10 a', "ten A",                 a[10]);
ATC('Cls .proto start0 array trans ref 10 b', "ten A",                 b[10]);
ATC('Cls .proto start0 array immed ref 10 c', "ten C",                 c[10]);
ATC('Fcn .proto object             ref 10 d', "ten Dpre",          d_pre[10]);
ATC('Fcn .proto object             ref 10 e', "ten Epre",          e_pre[10]);
ATC('Fcn .proto start0 array trans ref 10 d', "ten A",            d_post[10]);
ATC('Fcn .proto start0 array immed ref 10 e', "ten Epost",        e_post[10]);
ATC('Fcn .proto object             ref 10 f', "ten F",                 f[10]);

ATC('Arr .proto start0 array       ref 1k a', undefined,               a[1000]);
ATC('Cls .proto start0 array trans ref 1k b', undefined,               b[1000]);
ATC('Cls .proto start0 array immed ref 1k c', undefined,               c[1000]);
ATC('Fcn .proto object             ref 1k d', undefined,           d_pre[1000]);
ATC('Fcn .proto object             ref 1k e', undefined,           e_pre[1000]);
ATC('Fcn .proto start0 array trans ref 1k d', undefined,          d_post[1000]);
ATC('Fcn .proto start0 array immed ref 1k e', undefined,          e_post[1000]);
ATC('Fcn .proto object             ref 1k f', undefined,               f[1000]);

Array   .prototype[1000] = "thou A";
C       .prototype[1000] = "thou C";
C_sparse.prototype[1000] = "thou C_sparse";
D_prototype_pre[1000]    = "thou Dpre";
E_prototype_pre[1000]    = "thou Epre";
E       .prototype[1000] = "thou Epost";
F       .prototype[1000] = "thou F";

// forcing D to be sparse without clobbering index 1000
D.prototype[2000]     = "twothou D";

ATC('Arr .proto sparse array        named a', "named A",               a["n"]);
ATC('Cls .proto sparse array trans  named b', "named A",               b["n"]);
ATC('Cls .proto sparse array immed  named c', "named C",               c["n"]);
ATC('Fcn .proto object              named d', "named Dpre",        d_pre["n"]);
ATC('Fcn .proto sparse array trans  named d', "named A",          d_post["n"]);
ATC('Fcn .proto object              named e', "named Epre",        e_pre["n"]);
ATC('Fcn .proto sparse array immed  named e', "named Epost",      e_post["n"]);
ATC('Fcn .proto object              named f', "named F",               f["n"]);

ATC('Arr .proto sparse array       ref  1 a', "one A",                 a[1]);
ATC('Arr .proto+self sparse        ref  1 a', "one A",          a_sparse[1]);
ATC('Cls .proto sparse array trans ref  1 b', "one A",                 b[1]);
ATC('Cls .proto+self sparse trans  ref  1 b', "one A",          b_sparse[1]);
ATC('Cls .proto sparse array immed ref  1 c', "one C",                 c[1]);
ATC('Cls .proto+self sparse immed  ref  1 c', "one C_sparse",   c_sparse[1]);
ATC('Fcn .proto object             ref  1 d', "one Dpre",          d_pre[1]);
ATC('Fcn .proto sparse array trans ref  1 d', "one A",            d_post[1]);
ATC('Fcn .proto object             ref  1 e', "one Epre",          e_pre[1]);
ATC('Fcn .proto sparse array immed ref  1 e', "one Epost",        e_post[1]);
ATC('Fcn .proto object             ref  1 f', "one F",                 f[1]);

ATC('Arr .proto sparse array       ref 10 a', "ten A",                 a[10]);
ATC('Arr .proto+self sparse        ref 10 a', "ten A",          a_sparse[10]);
ATC('Cls .proto sparse array trans ref 10 b', "ten A",                 b[10]);
ATC('Cls .proto+self sparse trans  ref 10 b', "ten A",          b_sparse[10]);
ATC('Cls .proto sparse array immed ref 10 c', "ten C",                 c[10]);
ATC('Cls .proto+self sparse immed  ref 10 c', "ten C_sparse",   c_sparse[10]);
ATC('Fcn .proto object             ref 10 d', "ten Dpre",          d_pre[10]);
ATC('Fcn .proto sparse array trans ref 10 d', "ten A",            d_post[10]);
ATC('Fcn .proto object             ref 10 e', "ten Epre",          e_pre[10]);
ATC('Fcn .proto sparse array immed ref 10 e', "ten Epost",        e_post[10]);
ATC('Fcn .proto object             ref 10 f', "ten F",                 f[10]);

ATC('Arr .proto sparse array       ref 1k a', "thou A",                a[1000]);
ATC('Arr .proto+self sparse        ref 1k a', "thou A",         a_sparse[1000]);
ATC('Cls .proto sparse array trans ref 1k b', "thou A",                b[1000]);
ATC('Cls .proto+self sparse trans  ref 1k b', "thou A",         b_sparse[1000]);
ATC('Cls .proto sparse array immed ref 1k c', "thou C",                c[1000]);
ATC('Cls .proto+self sparse immed  ref 1k c', "thou C_sparse",  c_sparse[1000]);
ATC('Fcn .proto object             ref 1k d', "thou Dpre",         d_pre[1000]);
ATC('Fcn .proto object             ref 1k e', "thou Epre",         e_pre[1000]);
ATC('Fcn .proto sparse array trans ref 1k d', "thou A",           d_post[1000]);
ATC('Fcn .proto sparse array immed ref 1k e', "thou Epost",       e_post[1000]);
ATC('Fcn .proto object             ref 1k f', "thou F",                f[1000]);





