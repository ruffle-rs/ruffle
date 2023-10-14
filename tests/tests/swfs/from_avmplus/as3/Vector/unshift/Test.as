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
 File Name:    unshift.as
 Description:  unshift(object,...items)
 inserts the values in items as new vector elements at the start of object, such
 that their order within the vector elements of object is the same as the order in which
 they appear in items. Existing vector elements in object are shifted upward in the index range
 and the length property of object is updated.
 *
 */
// var SECTION="";
// var VERSION = "ECMA_1";



var v1=new Vector.<int>();
v1.unshift();
Assert.expectEq(
  "unshift empty vector with no items still empty",
  "",
  v1.toString());

var v1= new Vector.<*>(5, true);
var errormsg="";
try {
  v1.unshift({});
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "unshift object vector with fixed length",
  "RangeError: Error #1126",
  Utils.parseError(errormsg,"RangeError: Error #1126".length));

var v1=new Vector.<int>();
v1.unshift(10);
Assert.expectEq(
  "unshift empty vector with single item",
  "10",
  v1.toString());

var v1=new Vector.<int>();
v1[0]=10;
v1.unshift(11);
Assert.expectEq(
  "unshift single element vector with single item",
  "11,10",
  v1.toString());

var v1=new Vector.<int>();
for (var i=0;i<10;i++) v1[i]=i;
v1.unshift(11);
Assert.expectEq(
  "unshift small vector with single item",
  "11,0,1,2,3,4,5,6,7,8,9",
  v1.toString());

var v1=new Vector.<int>(3,true);
v1[0]=10; v1[1]=11; v1[2]=12;
var errormsg="";
try {
  v1.unshift(9);
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "unshift single element vector with single item",
  "RangeError: Error #1126",
  Utils.parseError(errormsg,"RangeError: Error #1126".length));

// bug: https://bugzilla.mozilla.org/show_bug.cgi?id=469377
var strVector = new Vector.<String>;
strVector.push("Carol", "Justine");
strVector.unshift("Betty");
Assert.expectEq(
  "Vector.<String>.unshift()",
  "Betty,Carol,Justine",
  strVector.toString());
