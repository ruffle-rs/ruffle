/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import StaticFunctionBody.*;
import com.adobe.test.Assert;

class TestObjNoPackage{
    static function noReturnNoParams() { return "noReturnNoParams"; }
    static function noReturnParams(s:String, b:Boolean) { return s; }
    static function noReturnCustomParam(c:Custom) { return new Custom(); }
    static function returnNoParams():String { return "returnNoParams"; }
    static function returnParams(s:String, b:Boolean):String { return s; }
    static function returnCustomNoParams():Custom { return new Custom(); }
}

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
Assert.expectEq( "TESTOBJ.noReturnCustomParams()", "[object Custom]", String(TESTOBJ.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TESTOBJ.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TESTOBJ.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TESTOBJ.returnCustomNoParams()) );

// outside package inside class
Assert.expectEq( "TESTOBJ.noReturnNoParams()", "noReturnNoParams", TestObjNoPackage.noReturnNoParams() );
Assert.expectEq( "TESTOBJ.noReturnParams(s,b)", "this is a test", TestObjNoPackage.noReturnParams(s,b) );
Assert.expectEq( "TESTOBJ.noReturnCustomParams()", "[object Custom]", String(TestObjNoPackage.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TestObjNoPackage.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TestObjNoPackage.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TestObjNoPackage.returnCustomNoParams()) );


              // displays results.
