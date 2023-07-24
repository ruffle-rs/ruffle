/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 Description:  The elements of this object are converted to strings and
 these strings are then concatenated, separated by comma
 characters. The result is the same as if the built-in join
 method were invoiked for this object with no argument.
 */

// var SECTION = "Vector";
// var VERSION = "Bug 678983";


function g(x) : Number { return x }
var err:String = "";

// -Dinterp avmplus::TypedVectorObject<TLIST>::checkWriteIndex_u(uint32_t) const
// -Ojit avmplus::TypedVectorObject<TLIST>::checkWriteIndex_d(double) const

err = "???"
try {
    var v:Vector.<*> = new Vector.<*>(10,true);
    var x:Number;
    x = 90;
    v[g(x)/10] = 'p';
    x = 100;
    v[g(x)/10] = 'p';
} catch (e) {
    err = e.toString();
}
// avmplus::TypedVectorObject<TLIST>::checkWriteIndex_d(double) const
Assert.expectEq("Vector fixed RangeError",
  "RangeError: Error #1125",
  err.substring(0,23));


err = "???"
try {
    var v:Vector.<*> = new Vector.<*>(10,true);
    var x:Number;
    x = 90;
    v[g(x)/10.1] = 'p';
} catch (e) {
    err = e.toString();
}
// avmplus::TypedVectorObject<TLIST>::checkWriteIndex_d(double) const
Assert.expectEq("Vector fixed RangeError: double(index_i) != index",
  "RangeError: Error #1125",
  err.substring(0,23));

err = "???"
try {
    var v:Vector.<*> = new Vector.<*>(10,true);
    var x:Number;
    x = 90;
    v[-g(x)/10] = 'p';
} catch (e) {
    err = e.toString();
}
// avmplus::TypedVectorObject<TLIST>::checkWriteIndex_d(double) const
Assert.expectEq("Vector fixed RangeError: index_i < 0",
  "RangeError: Error #1125",
  err.substring(0,23));


err = "???"
try {
    var v:Vector.<*> = new Vector.<*>(10,true);
    var x:Number;
    x = 90;
    var foo = v[g(x)/10];
    x = 100;
    var foo = v[g(x)/10];
} catch (e) {
    err = e.toString();
}
Assert.expectEq("Vector fixed RangeError checkReadIndex_d",
  "RangeError: Error #1125",
  err.substring(0,23));
