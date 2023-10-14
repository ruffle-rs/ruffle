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
 File Name:    some.as
 Description:  some(object,checker,thisobj=)
 calls checker on every vector element in object in increasing numerical index order,
 stopping as soon as checker returns a true value.
 checker is called with three arguments: the property value, the property index, and the object
 itself.  The thisobj is used as the this object in the call.
 returns true when checker returns a true value, otherwise returns false if all the calls to checker
 return false values.
 *
 */
// var SECTION="";
// var VERSION = "ECMA_1";


var msg;
function checker1(value,index,obj) {
  msg+="(value="+value+",index="+index+",object=["+obj+"])";
  if (value=='t')
    return true;
  return false;
}

var v1=new Vector.<int>();
var errormsg="";
try {
  result=v1.some();
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "some no checker",
  "ArgumentError: Error #1063",
  Utils.parseError(errormsg,"ArgumentError: Error #1063".length));


var checker2="a string";
var v1=new Vector.<int>();
v1.push(1);
var errormsg="";
try {
  result=v1.some(checker2);
} catch (e) {
  errormsg=e.toString();
}
Assert.expectEq(
  "some checker not a function",
  "TypeError: Error #1034",
  Utils.parseError(errormsg,"TypeError: Error #1034".length));

var msg="";
var v1=new Vector.<int>();
var result=v1.some(checker1);
Assert.expectEq(
  "some empty vector result",
  false,
  result);
Assert.expectEq(
  "some empty vector message empty",
  "",
  msg);
var msg="";
var v1=new Vector.<String>();
v1[0]='a';v1[1]='b';v1[2]='c';
var result=v1.some(checker1);
Assert.expectEq(
  "some small vector result",
  false,
  result);
Assert.expectEq(
  "some small vector message",
  "(value=a,index=0,object=[a,b,c])(value=b,index=1,object=[a,b,c])(value=c,index=2,object=[a,b,c])",
  msg);

var msg="";
var v1=new Vector.<String>();
v1[0]='a';v1[1]='b';v1[2]='t';v1[3]='c';v1[4]='d';
var result=v1.some(checker1);
Assert.expectEq(
  "some small vector result with a true",
  true,
  result);
Assert.expectEq(
  "some small vector message with a true",
  "(value=a,index=0,object=[a,b,t,c,d])(value=b,index=1,object=[a,b,t,c,d])(value=t,index=2,object=[a,b,t,c,d])",
  msg);


function ninetyNinePointNineNine(value,index,obj) {
  if (value == 99.99) {
    return true;
  }
  return false;
}

var v2 = Vector.<Number>([1.1, 3.1415, 33.33333, 99.99, 100.0000001]);
var v3 = Vector.<Number>([1.1, 3.1415, 33.33333, 99.9901, 100.0000001]);

Assert.expectEq("Verify a value is in a Number vector",
  true,
  v2.some(ninetyNinePointNineNine));

Assert.expectEq("Verify a value is not in a Number vector",
  false,
  v3.some(ninetyNinePointNineNine));

function uint_maxvalue(value,index,obj) {
  if (value == uint.MAX_VALUE) {
    return true;
  }
  return false;
}

var v2 = Vector.<uint>([1, 3, uint.MAX_VALUE, 99, 100]);
var v3 = Vector.<uint>([1, 3, uint.MIN_VALUE, 99, 100]);

Assert.expectEq("Verify a value is in a uint vector",
  true,
  v2.some(uint_maxvalue));

Assert.expectEq("Verify a value is not in a uint vector",
  false,
  v3.some(uint_maxvalue));

var myObj = {x:33};
var someFunc = function (value,index,obj) {
  if (this.x == 33) {
    return true;
  }
  return false;
}
Assert.expectEq("Test thisObject param",
  true,
  v3.some(someFunc, myObj));
