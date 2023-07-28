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
 File Name:    foreach.es
 Description:  foreach(object,eacher,thisobj)
 calls checker on every vector element of object in increasing numerical index order,
 collecting all the vector elements for which checker returns a value.
 checker is called with three arguments, the property value, the property index, the object itself.
 the thisobj is used as the this object in the call.
 returns a new vector object containing the elements that were collected in the order they were
 collected.
 */
// var SECTION="";
// var VERSION = "ECMA_1";



function eacher(value,index,obj) {
    result+="("+value+":"+index+")";
}
var bad_eacher="astring";

var v1=new Vector.<int>();
var errormsg="";
try {
    var result=v1.forEach();
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq(    "forEach eacher is undefined",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));

var v1=new Vector.<int>();
for (var i=0;i<3;i++) v1[i]=i;
var errormsg="";
try {
    var result=v1.forEach(bad_eacher);
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq("forEach eacher is not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

var v1=new Vector.<String>();
for (var i=0;i<3;i++) v1[i]="s"+i;
var result="";
v1.forEach(eacher);
Assert.expectEq(    "forEach simple vector",
  "(s0:0)(s1:1)(s2:2)",
  result);

function double(value,index,obj) {

}

var v1=Vector.<uint>([4560,9120,13680]);
var result="";
v1.forEach(eacher);
Assert.expectEq(    "forEach simple uint vector",
  "(4560:0)(9120:1)(13680:2)",
  result);

var v1=Vector.<Number>([4560,9120,13680]);
var result="";
v1.forEach(eacher);
Assert.expectEq(    "forEach simple Number vector",
  "(4560:0)(9120:1)(13680:2)",
  result);

var v1=Vector.<int>([4560,9120,13680]);
var result="";
v1.forEach(eacher);
Assert.expectEq(    "forEach simple uint vector",
  "(4560:0)(9120:1)(13680:2)",
  result);
