/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
public class Test {}
}


import com.adobe.test.Assert;
import com.adobe.test.Utils;

/**
 File Name:          initializerExpressions.as
 ECMA Section:       n/a
 Description:

 Test Vector intitializer expressions.

 Author:             tharwood@adobe.com
 Date:               27 March 2009
 */
// var SECTION=""
// var VERSION=""


var v = new<int>[1,2];

Assert.expectEq(
  "Initialize a variable with a Vector",
  "1,2",
  v.toString());

var v2:Vector.<int> = new<int>[3,4];

Assert.expectEq(
  "Initialize a typed variable with a Vector",
  "3,4",
  v2.toString());

var msg="no exception";
try {
  var v3:Vector.<*> = new<int>[49];
} catch ( ex )
{
  msg = ex.toString();
}

Assert.expectEq(
  "[neg]Initialize a typed variable with a Vector",
  "TypeError: Error #1034",
  Utils.parseError(msg,"TypeError: Error #1034".length));

Assert.expectEq(
  "pass Vector initializer as parameter",
  2,
  getLength(new <*>[new Object(),3.14159,]));

Assert.expectEq(
  "call Vector initializer method",
  3,
  new<int>[1,2,3].length);

Assert.expectEq(
  "compare Vector initializers",
  false,
  new<int>[1,2] == new<int>[1,2]);

Assert.expectEq(
  "compare Vector initializers - ne",
  true,
  new<int>[1,2] != new<int>[1,2]);

Assert.expectEq(
  "compare Vector initializers - ne",
  true,
  new<int>[1,2] != new<*>[1,2]);

Assert.expectEq(
  "add scalar to Vector",
  "1,2,34",
  new<int>[1,2,3]+4);

Assert.expectEq(
  "add Vector to scalar",
  "61,2,3",
  6+ new<int>[1,2,3]);


Assert.expectEq(
  "subtract scalar from Vector",
  NaN,
  new<int>[1,2,3]-4);

Assert.expectEq(
  "subtract Vector from scalar",
  NaN,
  4 - new<int>[1,2,3]);

Assert.expectEq(
  "multiply scalar by Vector",
  NaN,
  new<int>[1,2,3]*4);

Assert.expectEq(
  "multiply Vector by scalar",
  NaN,
  4 * new<int>[1,2,3]);

Assert.expectEq(
  "divide scalar by Vector",
  NaN,
  new<int>[1,2,3]/4);

Assert.expectEq(
  "divide Vector by scalar",
  NaN,
  4 / new<int>[1,2,3]);

Assert.expectEq(
  "typeof Vector initializer",
  "object",
  typeof(new<int>[1,2]));

Assert.expectEq(
  "select element from Vector initializer",
  3,
  new<int>[1,2,3,4][2]);

Assert.expectEq(
  "select element from Vector initializer",
  3,
  new<int>[1,2,3,4]["2.00"]);

Assert.expectEq(
  "assign to element from Vector initializer",
  7,
  new<int>[1,2,3,4][2] = 7);


Assert.expectEq(
  "stringify Vector initializer",
  "1,2,3",
  String(new<int>[1,2,3]));


Assert.expectEq(
  "delete Vector initializer property",
  false,
  delete new<int>[1,2,3].length);

Assert.expectEq(
  "initializer fixed property is false",
  false,
  new <int>[4,5,6,7,3,5,6,7,8].fixed);

Assert.expectEq(
  "length property returns expected value",
  10,
  new <Number> [0,1,2,3,4,5,6,7,8,9].length);

// length is used here since comparing the string is a pain
Assert.expectEq(
  "xml elements can be used in a vector literal",
  28,
  new<XML>[<myXml><item1/></myXml>,<myXml2></myXml2>].toString().length);

Assert.expectEq(
  "pop element from Vector initializer",
  "popped",
  new<String>['not this one', 'not this either','popped'].pop());

Assert.expectEq(
  "pop empty element from Vector initializer",
  null,
  new<String>['not this one', 'not this either','popped',null,].pop());

Assert.expectEq(
  "push element into Vector initializer",
  6,
  new<Number>[0.3,.56,.12,3.14].push(4500,.0001));

Assert.expectEq(
  "push nothing into Vector initializer",
  1,
  new<String>['hello'].push());

Assert.expectEq(
  "push null into Vector initializer",
  2,
  new<String>['hello'].push(null));

Assert.expectEq(
  "shift element from Vector initializer",
  uint(15e23),
  new<uint>[15e23,6,7].shift());

Assert.expectEq(
  "shift empty element from Vector initializer",
  null,
  new<String>[null,'4','5'].shift());

Assert.expectEq(
  "unshift element into Vector initializer",
  4,
  new<String>['3','4','5'].unshift('hello'));

Assert.expectEq(
  "unshift empty element into Vector initializer",
  3,
  new<String>['3','4','5'].unshift());

Assert.expectEq(
  "splice elements in Vector initializer",
  "3,4,5,6",
  new<int>[0,1,2,3,4,5,6,7,8,9].splice(3,4,101,102,103).toString());

var vSplice1:Vector.<int> = new<int>[0,1,2,3,4,5,6,7,8,9]
vSplice1.splice(3,4,101,102,103);
Assert.expectEq(
  "splice elements into Vector using comma seperated list",
  "0,1,2,101,102,103,7,8,9",
  vSplice1.toString());

var vSplice2:Vector.<int> = new<int>[0,1,2,3,4,5,6,7,8,9]
vSplice2.splice(3,4,101,102,103);
Assert.expectEq(
  "splice elements into Vector using vector initializer",
  "0,1,2,101,102,103,7,8,9",
  vSplice2.toString());


/* Not currently supported
var vSplice3:Vector.<int> = new<int>[0,1,2,3,4,5,6,7,8,9]
vSplice3.splice(3,4,[101,102,103]);
Assert.expectEq(
    "splice elements into Vector using array",
    "0,1,2,101,102,103,7,8,9",
    vSplice3.toString());
*/

Assert.expectEq(
  "create vector with one element and trailing comma in initializer",
  1,
  new<int>[333,].length);

Assert.expectEq(
  "create vector with null element and trailing comma in the initializer",
  1,
  new<Object>[null,].length);

Assert.expectEq(
  "create vector with undefined element and trailing comma in the initializer",
  1,
  new<Object>[undefined,].length);


function getLength(x:Vector.<*>):int
{
  return x.length;
}