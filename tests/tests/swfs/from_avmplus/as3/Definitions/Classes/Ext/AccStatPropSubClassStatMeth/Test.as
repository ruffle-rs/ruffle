/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "Access static property of base class from subclass";       // Provide ECMA section title or a description
var BUGNUMBER = "";


/**
 * Calls to Assert.expectEq here. Assert.expectEq is a function that is defined
 * in shell.js and takes three arguments:
 * - a string representation of what is being tested
 * - the expected result
 * - the actual result
 *
 * For example, a test might look like this:
 *
 * var helloWorld = "Hello World";
 *
 * Assert.expectEq(
 * "var helloWorld = 'Hello World'",   // description of the test
 *  "Hello World",                     // expected result
 *  helloWorld );                      // actual result
 *
 */

import StaticPropertyPackage.*;

import com.adobe.test.Assert;

// ********************************************
// Access the static property via BaseClass.x
// ********************************************
Assert.expectEq( "*** Access the static property via base class ***", 1, 1 );
Assert.expectEq( "BaseClass.i = 12, BaseClass.i", 12, (BaseClass.i = 12, BaseClass.i) );

// ********************************************
// Access the static property via sub class,
// using unadorned "x"
// ********************************************
Assert.expectEq( "*** Access the static property via sub class using unadorned property name ***", 1, 1 );
Assert.expectEq( "AccStatPropSubClassStatMeth.getInt()", BaseClass.i, AccStatPropSubClassStatMeth.getInt() );

// ********************************************
// Access the static property via sub class,
// using unadorned "BaseClass.x"
// ********************************************
Assert.expectEq( "*** Access the static property via sub class using unadorned property name ***", 1, 1 );
Assert.expectEq( "AccStatPropSubClassStatMeth.getBaseInt()", BaseClass.i, AccStatPropSubClassStatMeth.getBaseInt() );




              // displays results.
