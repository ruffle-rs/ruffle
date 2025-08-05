/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import EmptyFunctionBody.*;
import com.adobe.test.Assert;

class EmptyFunctionBodyClass {
    function EmptyFunctionBodyClass() {}
    function noReturnNoParams() { return "noReturnNoParams"; }
    function noReturnParams(s:String, b:Boolean) { return s; }
    function noReturnCustomParam(c:Custom) { return new Custom(); }
    function returnNoParams():String { return "returnNoParams"; }
    function returnParams(s:String, b:Boolean):String { return s; }
    function returnCustomNoParams():Custom { return new Custom(); }
}

function noReturnNoParamsNoPackage() { return "noReturnNoParams"; }
function noReturnParamsNoPackage(s:String, b:Boolean) { return s; }
function noReturnCustomParamNoPackage(c:Custom) { return new Custom(); }
function returnNoParamsNoPackage():String { return "returnNoParams"; }
function returnParamsNoPackage(s:String, b:Boolean):String { return s; }
function returnCustomNoParamsNoPackage():Custom { return new Custom(); }


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Body Parameter/Result Type";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ;
var s:String = new String("this is a test");
var b:Boolean = new Boolean(true);
var c:Custom = new Custom();

// inside class inside package
TESTOBJ = new TestObj();
Assert.expectEq( "TESTOBJ.noReturnNoParams()", "noReturnNoParams", TESTOBJ.noReturnNoParams() );
Assert.expectEq( "TESTOBJ.noReturnParams(s,b)", "this is a test", TESTOBJ.noReturnParams(s,b) );
//Function returns a new Custom object therefore must be compared using strings.
Assert.expectEq( "TESTOBJ.noReturnCustomParams(c)", "[object Custom]", String(TESTOBJ.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TESTOBJ.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TESTOBJ.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TESTOBJ.returnCustomNoParams()) );

// inside package outside of class
Assert.expectEq( "noReturnNoParams()", "noReturnNoParams", noReturnNoParams() );
Assert.expectEq( "noReturnParams(s,b)", "this is a test", noReturnParams(s,b) );
Assert.expectEq( "noReturnCustomParams()", "[object Custom]", String(noReturnCustomParam(c)) );
Assert.expectEq( "returnNoParams()", "returnNoParams", returnNoParams() );
Assert.expectEq( "returnParams(s,b)", "this is a test", returnParams(s,b) );
Assert.expectEq( "returnCustomNoParams()", "[object Custom]", String(returnCustomNoParams()) );

// outside package inside class
TESTOBJ = new EmptyFunctionBodyClass();
Assert.expectEq( "TESTOBJ.noReturnNoParams()", "noReturnNoParams", TESTOBJ.noReturnNoParams() );
Assert.expectEq( "TESTOBJ.noReturnParams(s,b)", "this is a test", TESTOBJ.noReturnParams(s,b) );
Assert.expectEq( "TESTOBJ.noReturnCustomParams()", "[object Custom]", String(TESTOBJ.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TESTOBJ.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TESTOBJ.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TESTOBJ.returnCustomNoParams()) );

// outside package and outside class
Assert.expectEq( "noReturnNoParamsNoPackage()", "noReturnNoParams", noReturnNoParamsNoPackage() );
Assert.expectEq( "noReturnParamsNoPackage(s,b)", "this is a test", noReturnParamsNoPackage(s,b) );
Assert.expectEq( "noReturnCustomParamsNoPackage()", "[object Custom]", String(noReturnCustomParamNoPackage(c)) );
Assert.expectEq( "returnNoParamsNoPackage()", "returnNoParams", returnNoParamsNoPackage() );
Assert.expectEq( "returnParamsNoPackage(s,b)", "this is a test", returnParamsNoPackage(s,b) );
Assert.expectEq( "returnCustomNoParamsNoPackage()", "[object Custom]", String(returnCustomNoParamsNoPackage()) );

              // displays results.
