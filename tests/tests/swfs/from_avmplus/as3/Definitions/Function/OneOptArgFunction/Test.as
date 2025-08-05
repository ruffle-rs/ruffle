/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import OneOptArgFunction.*
import com.adobe.test.Assert;

function returnStringNoPackage(s:String = "outside package and outside class",... rest):String { return s; }
function returnBooleanNoPackage(b:Boolean = true,... rest):Boolean { return b; }
function returnNumberNoPackage(n:Number = 10,... rest):Number { return n; }

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Body Parameter/Result Type";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ = new TestObj();
var TESTOBJ1 = new OneOptArgFunctionClass();
var s:String = new String("this is a test");
var b:Boolean = new Boolean(true);

//Optional String argument
// inside class inside package
Assert.expectEq( "TESTOBJ.returnString()", "inside class inside package", TESTOBJ.returnString() );

// inside package outside of class
Assert.expectEq( "returnString()", "inside package outside of class", returnString() );

// outside package inside class
Assert.expectEq( "TESTOBJ1.returnString()", "outside package inside class", TESTOBJ1.returnString() );

// outside package and outside class
Assert.expectEq( "returnStringNoPackage()", "outside package and outside class", returnStringNoPackage("outside package and outside class",true) );



//Optional Boolean argument
// inside class inside package
Assert.expectEq( "TESTOBJ.returnBoolean()", true, TESTOBJ.returnBoolean() );

// inside package outside of class
Assert.expectEq( "returnBoolean()", true, returnBoolean() );

// outside package inside class
Assert.expectEq( "TESTOBJ1.returnBoolean()", true, TESTOBJ1.returnBoolean() );

// outside package and outside class
Assert.expectEq( "returnBooleanNoPackage()", true, returnBooleanNoPackage() );


//Optional Number argument
// inside class inside package
Assert.expectEq( "TESTOBJ.returnNumber()", 10, TESTOBJ.returnNumber() );

// inside package outside of class
Assert.expectEq( "returnNumber()", 12, returnNumber() );

// outside package inside class
Assert.expectEq( "TESTOBJ1.returnNumber()", 9, TESTOBJ1.returnNumber(9,10) );

// outside package and outside class
Assert.expectEq( "returnNumberNoPackage()", 11, returnNumberNoPackage(11,"Hello") );


              // displays results.
