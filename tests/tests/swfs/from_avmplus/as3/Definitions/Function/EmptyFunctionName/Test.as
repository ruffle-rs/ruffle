/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import EmptyFunctionName.*;
import com.adobe.test.Assert;

class EmptyFunctionNameClass {
        // constructor
        function EmptyFunctionNameClass() { res = "EmptyName"; }

        // not the constructor but looks like it
        function emptyFunctionNameClass() { return "not the constructor" }

        function a1 () { return "a1"; }
        function a_1 () { return "a_1"; }
        function _a1 () { return "_a1"; }
        function __a1 () { return "__a1"; }
        function _a1_ () { return "_a1_"; }
        function __a1__ () { return "__a1__"; }
        function $a1 () { return "$a1"; }
        function a$1 () { return "a$1"; }
        function a1$ () { return "a1$"; }
        function A1 () { return "A1"; }
        function cases () { return "cases"; }
        function Cases () { return "Cases"; }
        function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
}

function b1 () { return "b1"; }
function b_1 () { return "b_1"; }
function _b1 () { return "_b1"; }
function __b1 () { return "__b1"; }
function _b1_ () { return "_b1_"; }
function __b1__ () { return "__b1__"; }
function $b1 () { return "$b1"; }
function b$1 () { return "b$1"; }
function b1$ () { return "b1$"; }
function B1 () { return "B1"; }
function bcases () { return "bcases"; }
function BCases () { return "BCases"; }
function abbcdefghijklmnopqrstuvwxyz0123456789$_ () { return "ball"; }


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

// outside class inside package
Assert.expectEq( "outside class inside package a1()", "a1", puba1() );
Assert.expectEq( "outside class inside package a_1()", "a_1", puba_1() );
Assert.expectEq( "outside class inside package function Name _a1()", "_a1", pub_a1() );
Assert.expectEq( "outside class inside package function Name __a1()", "__a1", pub__a1() );
Assert.expectEq( "outside class inside package function Name _a1_()", "_a1_", pub_a1_() );
Assert.expectEq( "outside class inside package function Name __a1__()", "__a1__", pub__a1__() );
Assert.expectEq( "outside class inside package function Name $a1()", "$a1", pub$a1() );
Assert.expectEq( "outside class inside package function Name a$1()", "a$1", puba$1() );
Assert.expectEq( "outside class inside package function Name a1$()", "a1$", puba1$() );
Assert.expectEq( "outside class inside package function Name C1()", "C1", pubC1() );
Assert.expectEq( "outside class inside package function Name cases()", "cases", pubcases() );

// inside class outside package
TESTOBJ = new EmptyFunctionNameClass();
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
Assert.expectEq( "inside class outside package function Name constructor different case", "not the constructor", TESTOBJ.emptyFunctionNameClass() );

// outside class outside package
Assert.expectEq( "outside class outside package b1()", "b1", b1() );
Assert.expectEq( "outside class outside package b_1()", "b_1", b_1() );
Assert.expectEq( "outside class outside package function Name _b1()", "_b1", _b1() );
Assert.expectEq( "outside class outside package function Name __b1()", "__b1", __b1() );
Assert.expectEq( "outside class outside package function Name _b1_()", "_b1_", _b1_() );
Assert.expectEq( "outside class outside package function Name __b1__()", "__b1__", __b1__() );
Assert.expectEq( "outside class outside package function Name $b1()", "$b1", $b1() );
Assert.expectEq( "outside class outside package function Name b$1()", "b$1", b$1() );
Assert.expectEq( "outside class outside package function Name b1$()", "b1$", b1$() );
Assert.expectEq( "outside class outside package function Name B1()", "B1", B1() );
Assert.expectEq( "outside class outside package function Name bcases()", "bcases", bcases() );
Assert.expectEq( "outside class outside package function Name BCases()", "BCases", BCases() );
Assert.expectEq( "outside class outside package function Name ball()", "ball", abbcdefghijklmnopqrstuvwxyz0123456789$_() );


              // displays results.
