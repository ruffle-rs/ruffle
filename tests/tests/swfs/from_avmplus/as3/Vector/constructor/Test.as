/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

/*
 * tests Vector.<type>(length,fixed) tests length and fixed parameter true|false
*/

// var SECTION = " ";
// var VERSION = "AS3";
// var TITLE   = "Vector.<type> constructor (length, fixed)";


var v1=new Vector.<uint>;
Assert.expectEq("constructor no arg no parens syntax",0,v1.length);
var v2=new Vector.<uint>();
Assert.expectEq("constructor no arg empty parens syntax",0,v2.length);

var v3:Vector.<uint>=new Vector.<uint>();
Assert.expectEq("constructor to typed var no arg empty parens syntax",0,v3.length);

var v4=new Vector.<uint>(100);
Assert.expectEq("constructor length parameters",100,v4.length);

// default value for fixed is false
var v5=new Vector.<uint>(10);
v5.push(10);
Assert.expectEq( "constructor fixed parameter default is false", 11, v5.length);

var v6=new Vector.<uint>(10,false);
v6.push(10);
Assert.expectEq( "constructor fixed parameter false produces unfixed vector", 11, v6.length);
var v7=new Vector.<uint>(10,true);
var errormsg="";
try {
    v7.push(10);
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq( "constructor fixed parameter set to true write beyond length-1 throws exception",
  "RangeError: Error #1126",
  Utils.parseError(errormsg,"RangeError: Error #1126".length));
Assert.expectEq( "constructor fixed parameter set to true length is unchanged", 10, v7.length);
var v8=new Vector.<uint>(10,true);
var errormsg="";
try {
    v8[10];
} catch (e) {
    errormsg=e.toString();
}

/*
// 2^30=1073741824
var v9=new Vector.<int>(1073741824);
Assert.expectEq( "constructor large vector", 1073741824, v9.length);
*/

// test basic types: already
var v10=new Vector.<uint>();
v10.push(10);
Assert.expectEq( "constructor type uint", 10, v10[0]);
var v11=new Vector.<int>();
v11.push(-10);
Assert.expectEq( "constructor type int", -10, v11[0]);
var v12=new Vector.<Number>();
v12.push(3.14);
Assert.expectEq( "constructor type Number", 3.14, v12[0]);
var v13=new Vector.<Boolean>();
v13.push(true);
Assert.expectEq( "constructor type Boolean", true, v13[0]);
var v14=new Vector.<String>();
v14.push("astring");
Assert.expectEq( "constructor type String", "astring", v14[0]);
class c1 { };
class c2 { };
class c3 extends c1 { };
var v15=new Vector.<c1>();
var c1inst=new c1();
v15.push(c1inst);
Assert.expectEq("constructor type custom class c1", c1inst, v15[0]);

var v16=new Vector.<c1>();
var c2inst=new c2();
var errormsg="";
try {
    v16.push(c2inst);
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq("constructor type custom class class mismatch",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

var c3inst=new c3();
var v17=new Vector.<c1>();
v17.push(new c3());
Assert.expectEq("constructor type custom class can caste to custom class", "[object c3]", v17[0].toString());

var v18=new Vector.<uint>();
v18.push(true);
Assert.expectEq("constructor type uint castes other types", 1, v18[0]);

function bug449468() {
    var v : Vector.<Vector.<Number>> = new Vector.<Vector.<Number>>(4);
    return v;
}

Assert.expectEq("Bug 449468: Crash with vector constructor in interp mode",
  "null,null,null,null",
  bug449468().toString()
);
