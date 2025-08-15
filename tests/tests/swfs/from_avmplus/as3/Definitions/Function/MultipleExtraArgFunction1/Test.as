/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import MultipleExtraArgFunction1.*
import com.adobe.test.Assert;

function returnRestNoPackage(... rest):Number { return rest.length; }


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Optional Argument test";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ = new TestObj();
var TESTOBJ1 = new MultipleExtraArgFunction1Class();

// inside class inside package
Assert.expectEq( "TESTOBJ.returnRest()", 6, TESTOBJ.returnRest(10,false,"hello",new Object(), [123],[345]) );

// inside package outside of class
Assert.expectEq( "returnRest()", 7, returnRest([10,11,12],false,"hello",new Object(), [123],[345],"hello") );

// outside package inside class
Assert.expectEq( "TESTOBJ1.returnRest()", 6, TESTOBJ1.returnRest(10,new Object(),"hello",new Object(), [123],[345]) );

// outside package and outside class
Assert.expectEq( "returnRestNoPackage()", 6, returnRestNoPackage(10,"str2","hello",new Object(), [123],[345]) );


              // displays results.
