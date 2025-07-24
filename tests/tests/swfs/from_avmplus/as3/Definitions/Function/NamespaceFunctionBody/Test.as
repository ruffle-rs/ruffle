/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import NamespaceFunctionBody.*;
import com.adobe.test.Assert;

// inside class, outside package
class NamespaceFunctionBodyClass {
    function NamespaceFunctionBodyClass() {}
    testns function noReturnNoParams() { return "noReturnNoParams"; }
    testns function noReturnParams(s:String, b:Boolean) { return s; }
    testns function noReturnCustomParam(c:Custom) { return new Custom(); }
    testns function returnNoParams():String { return "returnNoParams"; }
    testns function returnParams(s:String, b:Boolean):String { return s; }
    testns function returnCustomNoParams():Custom { return new Custom(); }
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Body Parameter/Result Type";       // Provide ECMA section title or a description
var BUGNUMBER = "";


use namespace testns;
var TESTOBJ;
var s:String = new String("this is a test");
var b:Boolean = new Boolean(true);
var c:Custom = new Custom();

// inside class, inside package
TESTOBJ = new TestObj();
Assert.expectEq( "TESTOBJ.noReturnNoParams()", "noReturnNoParams", TESTOBJ.noReturnNoParams() );
Assert.expectEq( "TESTOBJ.noReturnParams(s,b)", "this is a test", TESTOBJ.noReturnParams(s,b) );
Assert.expectEq( "TESTOBJ.noReturnCustomParams(c)", "[object Custom]", String(TESTOBJ.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TESTOBJ.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TESTOBJ.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TESTOBJ.returnCustomNoParams()) );

// inside class, outside package
TESTOBJ = new NamespaceFunctionBodyClass();
Assert.expectEq( "TESTOBJ.noReturnNoParams()", "noReturnNoParams", TESTOBJ.noReturnNoParams() );
Assert.expectEq( "TESTOBJ.noReturnParams(s,b)", "this is a test", TESTOBJ.noReturnParams(s,b) );
Assert.expectEq( "TESTOBJ.noReturnCustomParams()", "[object Custom]", String(TESTOBJ.noReturnCustomParam(c)) );
Assert.expectEq( "TESTOBJ.returnNoParams()", "returnNoParams", TESTOBJ.returnNoParams() );
Assert.expectEq( "TESTOBJ.returnParams(s,b)", "this is a test", TESTOBJ.returnParams(s,b) );
Assert.expectEq( "TESTOBJ.returnCustomNoParams()", "[object Custom]", String(TESTOBJ.returnCustomNoParams()) );


              // displays results.
