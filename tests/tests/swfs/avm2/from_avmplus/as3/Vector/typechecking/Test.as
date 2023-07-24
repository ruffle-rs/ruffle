/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/**
 File Name:    typechecking.as
 Description:
 verify runtime error when set or add mismatched types.
 *
 */
// var SECTION="";
// var VERSION = "ECMA_4";



// untyped
var vs1=new Vector.<String>();
// typed
var vs2:Vector.<String>=new Vector.<String>();
var v1:Vector.<Object>;

var err1="no error";
try {
  v1=vs2;
} catch (e) {
  err1=e.toString();
}

Assert.expectEq(
  "assign vector variable to mismatched builtin type, throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err1,"TypeError: Error #1034".length));

class A { }
class B extends A { }
class C { }

var v2:Vector.<A>=new Vector.<A>();
var v3:Vector.<B>=new Vector.<B>();
try {
  v2=v3;
} catch (e) {
  err3=e.toString();
}
Assert.expectEq(
  "assign vector variable to mismatched types, throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err3,"TypeError: Error #1034".length));


Assert.expectEq(
  "push inherited type into Vector is allowed",
  2,
  v2.push(new A(),new B()));

var err4="no error";
try {
  v2.push(new A(),new C());
} catch (e) {
  err4=e.toString();
}
Assert.expectEq(
  "push wrong type into Vector, throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err4,"TypeError: Error #1034".length));

var v4:Vector.<A>=new Vector.<A>();

Assert.expectEq(
  "unshift inherited types is allowed",
  2,
  v4.unshift(new A(),new B()));

Assert.expectEq(
  "unshift inherited types is allowed, verify toString",
  "[object A],[object B]",
  v4.toString());

var err5="no error";
var v5=new Vector.<A>();
try {
  v5.unshift(new A(),new B(),new C());
} catch (e) {
  err5=e.toString();
}
Assert.expectEq(
  "unshift wrong type into Vector, throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err5,"TypeError: Error #1034".length));

var v6:Vector.<A>=new Vector.<A>();
v6.push(new A(),new A());
var v6a:Vector.<B>=new Vector.<B>();
v6a.push(new B(),new B());
Assert.expectEq(
  "concat inherited types returns correct vector",
  "[object A],[object A],[object B],[object B]",
  v6.concat(v6a).toString());

var err7="no error";
var v7:Vector.<C>=new Vector.<C>();
v7.push(new C());
try {
  v6.concat(v7);
} catch (e) {
  err7=e.toString();
}
Assert.expectEq(
  "vector concat throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err7,"TypeError: Error #1034".length));

var v8:Vector.<A>=new Vector.<A>();
var err8="no error";
try {
  v8[0]=new C();
} catch(e) {
  err8=e.toString();
}
Assert.expectEq(
  "vector assignment on incorrect type throws runtime error",
  "TypeError: Error #1034",
  Utils.parseError(err8,"TypeError: Error #1034".length));
