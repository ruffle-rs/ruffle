/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import OneExtraArgFunction.*
import com.adobe.test.Assert;

function returnRestNoPackage(... rest):Number { return rest.length; }


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Optional Argument test";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ = new TestObj();
var TESTOBJ1 = new OneExtraArgFunctionClass();

// Pass a Number argument
// inside class inside package
Assert.expectEq( "Number TESTOBJ.returnRest()", 1, TESTOBJ.returnRest(10) );

// inside package outside of class
Assert.expectEq( "Number returnRest()", 1, returnRest(20) );

// outside package inside class
Assert.expectEq( "Number TESTOBJ1.returnRest()", 1, TESTOBJ1.returnRest(30) );

// outside package and outside class
Assert.expectEq( "Number returnRestNoPackage()", 1, returnRestNoPackage(40) );


// Pass a String argument
// inside class inside package
Assert.expectEq( "String TESTOBJ.returnRest()", 1, TESTOBJ.returnRest("Str1") );

// inside package outside of class
Assert.expectEq( "String returnRest()", 1, returnRest("Str2") );

// outside package inside class
Assert.expectEq( "String TESTOBJ1.returnRest()", 1, TESTOBJ1.returnRest("Str3") );

// outside package and outside class
Assert.expectEq( "String returnRestNoPackage()", 1, returnRestNoPackage("Str4") );


// Pass a Boolean argument
// inside class inside package
Assert.expectEq( "Boolean TESTOBJ.returnRest()", 1, TESTOBJ.returnRest(true) );

// inside package outside of class
Assert.expectEq( "Boolean returnRest()", 1, returnRest(false) );

// outside package inside class
Assert.expectEq( "Boolean TESTOBJ1.returnRest()", 1, TESTOBJ1.returnRest(true) );

// outside package and outside class
Assert.expectEq( "Boolean returnRestNoPackage()", 1, returnRestNoPackage(false) );


// Pass an Object argument
// inside class inside package
Assert.expectEq( "Object TESTOBJ.returnRest()", 1, TESTOBJ.returnRest(new Object()) );

// inside package outside of class
Assert.expectEq( "Object returnRest()", 1, returnRest(new Object()) );

// outside package inside class
Assert.expectEq( "Object TESTOBJ1.returnRest()", 1, TESTOBJ1.returnRest(new Object()) );

// outside package and outside class
Assert.expectEq( "Object returnRestNoPackage()", 1, returnRestNoPackage(new Object()) );

// Pass an Array argument
// inside class inside package
Assert.expectEq( "Array TESTOBJ.returnRest()", 1, TESTOBJ.returnRest([10,20,30]) );

// inside package outside of class
Assert.expectEq( "Array returnRest()", 1, returnRest([10,20,30]) );

// outside package inside class
Assert.expectEq( "Array TESTOBJ1.returnRest()", 1, TESTOBJ1.returnRest([10,20,30]) );

// outside package and outside class
Assert.expectEq( "Array returnRestNoPackage()", 1, returnRestNoPackage([10,20,30]) );


              // displays results.
