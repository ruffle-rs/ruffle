/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import PrivateFunctionName.*;
import com.adobe.test.Assert;

class PrivateFunctionNameClass {

    private function a1 () { return "a1"; }
    private function a_1 () { return "a_1"; }
    private function _a1 () { return "_a1"; }
    private function __a1 () { return "__a1"; }
    private function _a1_ () { return "_a1_"; }
    private function __a1__ () { return "__a1__"; }
    private function $a1 () { return "$a1"; }
    private function a$1 () { return "a$1"; }
    private function a1$ () { return "a1$"; }
    private function A1 () { return "A1"; }
    private function cases () { return "cases"; }
    private function Cases () { return "Cases"; }
    private function abcdefghijklmnopqrstuvwxyz0123456789$_ () { return "all"; }

    public function puba1 () { return a1(); }
    public function puba_1 () { return a_1(); }
    public function pub_a1 () { return _a1(); }
    public function pub__a1 () { return __a1(); }
    public function pub_a1_ () { return _a1_(); }
    public function pub__a1__ () { return __a1__(); }
    public function pub$a1 () { return $a1(); }
    public function puba$1 () { return a$1(); }
    public function puba1$ () { return a1$(); }
    public function pubA1 () { return A1(); }
    public function pubcases () { return cases(); }
    public function pubCases () { return Cases(); }
    public function puball () { return abcdefghijklmnopqrstuvwxyz0123456789$_(); }
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

// inside class outside package
TESTOBJ = new PrivateFunctionNameClass();
Assert.expectEq( "inside class outside package function Name a1()", "a1", TESTOBJ.puba1() );
Assert.expectEq( "inside class outside package function Name a_1()", "a_1", TESTOBJ.puba_1() );
Assert.expectEq( "inside class outside package function Name _a1()", "_a1", TESTOBJ.pub_a1() );
Assert.expectEq( "inside class outside package function Name __a1()", "__a1", TESTOBJ.pub__a1() );
Assert.expectEq( "inside class outside package function Name _a1_()", "_a1_", TESTOBJ.pub_a1_() );
Assert.expectEq( "inside class outside package function Name __a1__()", "__a1__", TESTOBJ.pub__a1__() );
Assert.expectEq( "inside class outside package function Name $a1()", "$a1", TESTOBJ.pub$a1() );
Assert.expectEq( "inside class outside package function Name a$1()", "a$1", TESTOBJ.puba$1() );
Assert.expectEq( "inside class outside package function Name a1$()", "a1$", TESTOBJ.puba1$() );
Assert.expectEq( "inside class outside package function Name A1()", "A1", TESTOBJ.pubA1() );
Assert.expectEq( "inside class outside package function Name cases()", "cases", TESTOBJ.pubcases() );
Assert.expectEq( "inside class outside package function Name Cases()", "Cases", TESTOBJ.pubCases() );
Assert.expectEq( "inside class outside package function Name all()", "all", TESTOBJ.puball() );


              // displays results.
