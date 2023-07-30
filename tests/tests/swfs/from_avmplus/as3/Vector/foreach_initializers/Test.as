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

var errormsg="";
try {
    var result=new <int>[].forEach();
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq(    "forEach eacher is undefined",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));

var errormsg="";
try {
    var result=new <int>[0,1,2].forEach(bad_eacher);
} catch (e) {
    errormsg=e.toString();
}
Assert.expectEq("forEach eacher is not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

var result="";
new <String>["s0","s1","s2"].forEach(eacher);
Assert.expectEq(    "forEach simple vector",
  "(s0:0)(s1:1)(s2:2)",
  result);

var i:int;
for (i in new <int> [1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8])
{}

Assert.expectEq("for-in loop",
  39,
  i);


var str:String = "";
for each (var o in new <Object>[1,2,3,"hello",'out',"there",true, false, 3.14159])
{
    str += o;
}

Assert.expectEq("for-each-in loop",
  "123helloouttheretruefalse3.14159",
  str);
