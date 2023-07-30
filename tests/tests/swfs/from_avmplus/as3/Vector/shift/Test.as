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
 File Name:    shift.as
 Description:  shift(object)
 removes the element called 0 in object, moves the element at index i+1 to index i,
 and decrements the length property of object by 1.
 *
 */
// var SECTION="";
// var VERSION = "ECMA_1";



var v1=new Vector.<int>();
Assert.expectEq(
  "shift on empty vector returns undefined",
  0,
  v1.shift());
Assert.expectEq(
  "shift on empty vector original vector is empty",
  "",
  v1.toString());

var v1=new Vector.<int>();
v1.push(10);
Assert.expectEq(
  "shift on single element vector returns element[0]",
  10,
  v1.shift());
Assert.expectEq(
  "shift on single element vector removes first element",
  "",
  v1.toString());

var v1=new Vector.<int>();
for (var i=0;i<5;i++) v1[i]=10+i;
Assert.expectEq(
  "shift on vector returns element[0]",
  10,
  v1.shift());
Assert.expectEq(
  "shift on vector removes first element",
  "11,12,13,14",
  v1.toString());

var v1=new Vector.<int>(10);
Assert.expectEq(
  "shift on initialized vector returns element[0]",
  0,
  v1.shift());
Assert.expectEq(
  "shift on initialized vector removes first element",
  "0,0,0,0,0,0,0,0,0",
  v1.toString());

var v1=new Vector.<int>(10,true);
for (var i=0;i<10;i++) v1[i]=10+i;

var errormsg="";
try {
  v1.shift();
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "shift on fixed vector throws exception",
  "RangeError: Error #1126",
  Utils.parseError(errormsg,"RangeError: Error #1126".length));

Assert.expectEq(
  "shift on fixed vector does not shift",
  "10,11,12,13,14,15,16,17,18,19",
  v1.toString());

Assert.expectEq("Shift string vector",
  "h",
  Vector.<String>(['h','e','l','l','o']).shift()
);

class TestClass {
  private var myVal:Object;
  public function TestClass(v:Object):void {
    myVal = v;
  }
  public function toString():String {
    return myVal.toString();
  }
}

Assert.expectEq("Shift custom vector class",
  "-Infinity",
  Vector.<TestClass>([new TestClass(-Infinity), new TestClass(55), new TestClass(789)]).shift().toString()
);
