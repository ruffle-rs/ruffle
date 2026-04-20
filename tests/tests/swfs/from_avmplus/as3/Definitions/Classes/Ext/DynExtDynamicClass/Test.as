/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import DynamicClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "dynamic extend Dynamic Class";       // Provide ECMA section title or a description
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


//**********************************************
// access default method of parent class
// from outside of class
//**********************************************

var EXTDCLASS = new DynExtDynamicClass();
var arr = new Array(1, 2, 3);

// ********************************************
// access default method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );


// ********************************************
// access default method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access default method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access default method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );


// ********************************************
// access default property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default property from method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubGetDPArray(arr)", arr, (EXTDCLASS.testSubGetDPArray(arr)) );


// ********************************************
// access default property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access default property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubGetDPArray(arr)", arr, (EXTDCLASS.testPrivSubGetDPArray(arr)) );


// ********************************************
// access default property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClass();
Assert.expectEq( "*** Access default property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubGetDPArray(arr)", arr, (EXTDCLASS.testFinSubGetDPArray(arr)) );

              // displays results.
