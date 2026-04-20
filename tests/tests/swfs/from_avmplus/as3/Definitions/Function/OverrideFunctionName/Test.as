/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import OverrideFunctionName.*;
import com.adobe.test.Assert;

// inside class outside package
class OverrideFunctionNameClassBase {
        // constructor
        function OverrideFunctionNameClassBase() {}

        // not the constructor but looks like it
        function overrideFunctionNameClass() { return null; }

        function a1 () { return null; }
        function a_1 () { return null; }
        function _a1 () { return null; }
        function __a1 () { return null; }
        function _a1_ () { return null; }
        function __a1__ () { return null; }
        function $a1 () { return null; }
        function a$1 () { return null; }
        function a1$ () { return null; }
        function A1 () { return null; }
        function cases () { return null; }
        function Cases () { return null; }
        function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return null; }
}

class OverrideFunctionNameClass extends OverrideFunctionNameClassBase {
        // constructor
        function OverrideFunctionNameClass() {}

        // not the constructor but looks like it
        override function overrideFunctionNameClass() { return "not the constructor" }

        override function a1 () { return "a1"; }
        override function a_1 () { return "a_1"; }
        override function _a1 () { return "_a1"; }
        override function __a1 () { return "__a1"; }
        override function _a1_ () { return "_a1_"; }
        override function __a1__ () { return "__a1__"; }
        override function $a1 () { return "$a1"; }
        override function a$1 () { return "a$1"; }
        override function a1$ () { return "a1$"; }
        override function A1 () { return "A1"; }
        override function cases () { return "cases"; }
        override function Cases () { return "Cases"; }
        override function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Names";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ;

// inside class inside package
TESTOBJ = new TestNameObj();
Assert.expectEq( "inside class inside package function Name a1()", "a1", TESTOBJ.a1() );
Assert.expectEq( "inside class inside package function Name a_1()", "a_1", TESTOBJ.a_1() );
Assert.expectEq( "inside class inside package function Name _a1()", "_a1", TESTOBJ._a1() );
Assert.expectEq( "inside class inside package function Name __a1()", "__a1", TESTOBJ.__a1() );
Assert.expectEq( "inside class inside package function Name _a1_()", "_a1_", TESTOBJ._a1_() );
Assert.expectEq( "inside class inside package function Name __a1__()", "__a1__", TESTOBJ.__a1__() );
Assert.expectEq( "inside class inside package function Name $a1()", "$a1", TESTOBJ.$a1() );
Assert.expectEq( "inside class inside package function Name a$1()", "a$1", TESTOBJ.a$1() );
Assert.expectEq( "inside class inside package function Name a1$()", "a1$", TESTOBJ.a1$() );
Assert.expectEq( "inside class inside package function Name A1()", "A1", TESTOBJ.A1() );
Assert.expectEq( "inside class inside package function Name cases()", "cases", TESTOBJ.cases() );
Assert.expectEq( "inside class inside package function Name Cases()", "Cases", TESTOBJ.Cases() );
Assert.expectEq( "inside class inside package function Name all()", "all", TESTOBJ.abcdefghijklmnopqrstuvwxyz0123456789$_() );
Assert.expectEq( "inside class inside package function Name constructor different case", "not the constructor", TESTOBJ.testNameObj() );

// inside class outside package
TESTOBJ = new OverrideFunctionNameClass();
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
Assert.expectEq( "inside class outside package function Name constructor different case", "not the constructor", TESTOBJ.overrideFunctionNameClass() );


              // displays results.
