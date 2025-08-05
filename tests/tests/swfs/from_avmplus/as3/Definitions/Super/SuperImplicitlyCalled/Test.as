/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "8.6.1 Constructor Methods";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Implicit SuperStatement";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

import SuperImplicitlyCalledPackage.*

import com.adobe.test.Assert;
Assert.expectEq( "initial counter", 0, SuperImplicitlyCalled.howManyObjects() );

var explicitCase = new SuperImplicitlyCalled1();
Assert.expectEq( "explicit counter", 1, SuperImplicitlyCalled.howManyObjects() );

var implicitCase = new SuperImplicitlyCalled2();
Assert.expectEq( "implicit counter", 2, SuperImplicitlyCalled.howManyObjects() );

var implicitCase = new SuperImplicitlyCalled2();
Assert.expectEq( "implicit counter with no constructor defined", 3, SuperImplicitlyCalled.howManyObjects() );

//
////////////////////////////////////////////////////////////////

              // displays results.
