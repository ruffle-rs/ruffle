/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 File Name:          reverse.as
 ECMA Section:       Vector.reverse()
 Description:

 The elements of the vector are rearranged so as to reverse their order.
 This object is returned as the result of the call.


 Note that the reverse function is intentionally generic; it does not require
 that its this value be an Array/Vector object. Therefore it can be transferred to other
 kinds of objects for use as a method. Whether the reverse function can be applied
 successfully to a host object is implementation dependent.

 Author:             christine@netscape.com
 Date:               7 october 1997
 */
// var SECTION = "";
// var VERSION = "ECMA_1";

Assert.expectEq(
  "Vector.<int>.prototype.reverse.length",
  0,
  Vector.<int>.prototype.reverse.length );
var v1:Vector.<int>=new Vector.<int>();
v1.reverse();
Assert.expectEq(
  "reverse empty vector",
  "",
  v1.toString());

var v1:Vector.<int>=new Vector.<int>();
v1[0]=111;
v1.reverse();
Assert.expectEq(
  "reverse vector length 1",
  "111",
  v1.toString());

var v1:Vector.<int>=new Vector.<int>();
for (var i=0;i<20;i++) v1[i]=i;
v1.reverse();
Assert.expectEq(
  "reverse vector of int",
  "19,18,17,16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0",
  v1.toString() );

var v2:Vector.<String>=new Vector.<String>();
v2[0]='one';v2[1]='two';v2[2]='three';
v2.reverse();
Assert.expectEq(
  "reverse vector of String",
  "three,two,one",
  v2.toString() );
