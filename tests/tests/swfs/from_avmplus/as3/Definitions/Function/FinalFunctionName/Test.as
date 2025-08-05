/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import FinalFunctionName.*;
import com.adobe.test.Assert;

class FinalFunctionNameClass {
    // constructor
    function TestNameObjNoPackage() { res = "EmptyName"; }

    // not the constructor but looks like it
    function testNameObjNoPackage() { return "not the constructor" }

    final function a1 () { return "a1"; }
    final function a_1 () { return "a_1"; }
    final function _a1 () { return "_a1"; }
    final function __a1 () { return "__a1"; }
    final function _a1_ () { return "_a1_"; }
    final function __a1__ () { return "__a1__"; }
    final function $a1 () { return "$a1"; }
    final function a$1 () { return "a$1"; }
    final function a1$ () { return "a1$"; }
    final function A1 () { return "A1"; }
    final function cases () { return "cases"; }
    final function Cases () { return "Cases"; }
    final function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Names";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ;

// inside class inside package
TESTOBJ = new TestNameObj();
Assert.expectEq( "inside class inside package function Name a1()", "a1", TESTOBJ.puba1() );
Assert.expectEq( "inside class inside package function Name a_1()", "a_1", TESTOBJ.puba_1() );
Assert.expectEq( "inside class inside package function Name _a1()", "_a1", TESTOBJ.pub_a1() );
Assert.expectEq( "inside class inside package function Name __a1()", "__a1", TESTOBJ.pub__a1() );
Assert.expectEq( "inside class inside package function Name _a1_()", "_a1_", TESTOBJ.pub_a1_() );
Assert.expectEq( "inside class inside package function Name __a1__()", "__a1__", TESTOBJ.pub__a1__() );
Assert.expectEq( "inside class inside package function Name $a1()", "$a1", TESTOBJ.pub$a1() );
Assert.expectEq( "inside class inside package function Name a$1()", "a$1", TESTOBJ.puba$1() );
Assert.expectEq( "inside class inside package function Name a1$()", "a1$", TESTOBJ.puba1$() );
Assert.expectEq( "inside class inside package function Name A1()", "A1", TESTOBJ.pubA1() );
Assert.expectEq( "inside class inside package function Name cases()", "cases", TESTOBJ.pubcases() );
Assert.expectEq( "inside class inside package function Name Cases()", "Cases", TESTOBJ.pubCases() );
Assert.expectEq( "inside class inside package function Name all()", "all", TESTOBJ.puball() );
Assert.expectEq( "inside class inside package function Name constructor different case", "not the constructor", TESTOBJ.pubTestConst() );

// inside class outside package
TESTOBJ = new FinalFunctionNameClass();
Assert.expectEq( "inside class outside package function Name a1()", "a1", TESTOBJ.a1() );
Assert.expectEq( "inside class outside package function Name a_1()", "a_1", TESTOBJ.a_1() );
Assert.expectEq( "inside class outside package function Name _a1()", "_a1", TESTOBJ._a1() );
Assert.expectEq( "inside class outside package function Name __a1()", "__a1", TESTOBJ.__a1() );
Assert.expectEq( "inside class outside package function Name _a1_()", "_a1_", TESTOBJ._a1_() );
Assert.expectEq( "inside class outside package function Name __a1__()", "__a1__", TESTOBJ.__a1__() );
Assert.expectEq( "inside class outside package function Name $a1()", "$a1", TESTOBJ.$a1() );
Assert.expectEq( "inside class outside package function Name a$1()", "a$1", TESTOBJ.a$1() );
Assert.expectEq( "inside class outside package function Name a1$()", "a1$", TESTOBJ.a1$() );
Assert.expectEq( "inside class outside package function Name A1()", "A1", TESTOBJ.A1() );
Assert.expectEq( "inside class outside package function Name cases()", "cases", TESTOBJ.cases() );
Assert.expectEq( "inside class outside package function Name Cases()", "Cases", TESTOBJ.Cases() );
Assert.expectEq( "inside class outside package function Name all()", "all", TESTOBJ.abcdefghijklmnopqrstuvwxyz0123456789$_() );
Assert.expectEq( "inside class outside package function Name constructor different case", "not the constructor", TESTOBJ.testNameObjNoPackage() );


              // displays results.
