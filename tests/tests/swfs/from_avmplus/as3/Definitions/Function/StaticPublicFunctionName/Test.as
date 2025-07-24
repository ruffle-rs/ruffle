/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import StaticPublicFunctionName.*;
import com.adobe.test.Assert;


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Names";       // Provide ECMA section title or a description
var BUGNUMBER = "";



var TESTOBJ = new TestNameObj();

// inside class inside package
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


              // displays results.
