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
// var TITLE   = "dynamic class extend dynamic class";       // Provide ECMA section title or a description
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


//*******************************************
//  access final method from
//  outside of class
//*******************************************

var EXTDCLASS = new DynExtDynamicClassFin();
var arr = new Array(1, 2, 3);

// ********************************************
// access final method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );



// ********************************************
// access final method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access final method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access final method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access final method from a private final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from private final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivFinSubArray(arr)", arr, (EXTDCLASS.testPrivFinSubArray(arr)) );

// ********************************************
// access final method from a virtual
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final method from virtual method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testVirSubArray(arr)", arr, (EXTDCLASS.testVirSubArray(arr)) );


// ********************************************
// access final property from outside
// the class
// ********************************************


// ********************************************
// access final property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final property from method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubGetDPArray(arr)", arr, (EXTDCLASS.testSubGetDPArray(arr)) );


// ********************************************
// access final property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access final property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubGetDPArray(arr)", arr, (EXTDCLASS.testPrivSubGetDPArray(arr)) );


// ********************************************
// access final property from
// private virtual method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassFin();
Assert.expectEq( "*** Access final property from private virtual method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivVirSubGetDPArray(arr)", arr, (EXTDCLASS.testPrivVirSubGetDPArray(arr)) );



              // displays results.
