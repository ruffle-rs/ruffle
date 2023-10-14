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
 File Name:    map.as
 Description:  map(object,mapper,thisobj)
 calls mapper on each vector element of object in increasing numerical index order, collecting
 the return values from mapper in a new vector object.
 mapper is called with three arguments: the property value, the property index, and object itself.
 The thisobj is used as the this object in the call.
 returns a new vector object where the vector element at index i is the value returned from the call
 to mapper on object[i].
 */
// var SECTION="";
// var VERSION = "ECMA_1";



function mapper1(value,index,obj) {
  return "("+value+":"+index+")";
}
var mapper2="a string";
function mapper3(value,index,obj) {
  return "("+this.message+")";
}
function mapper4(value,index,obj) {
  return value*value;
}

var errormsg="";
try {
  var result=new <int>[1].map();
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "map mapper is undefined",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));

var errormsg="";
try {
  var result=new<int>[1].map(mapper2);
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "map mapper is not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

Assert.expectEq(
  "map empty vector",
  "",
  new <int>[].map(mapper1).toString());

Assert.expectEq(
  "map small vector",
  "(a:0),(b:1),(c:2)",
  new<String>['a','b','c'].map(mapper1).toString());


Assert.expectEq(
  "map fixed size small vector",
  "(a:0),(b:1),(c:2)",
  new<String>['a','b','c'].map(mapper1).toString());

testobj=new Object();
testobj.message="testobj";
Assert.expectEq(   "map vector passing new object",
  "(testobj),(testobj)",
  new<String>['a','b'].map(mapper3,testobj).toString());

Assert.expectEq(   "map vector of int",
  "1,4,9,16",
  new<int>[1,2,3,4].map(mapper4).toString());
