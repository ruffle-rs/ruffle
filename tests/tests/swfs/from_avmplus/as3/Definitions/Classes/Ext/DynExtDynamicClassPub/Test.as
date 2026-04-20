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
// var TITLE   = "public extend Dynamic Class";       // Provide ECMA section title or a description
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

var EXTDCLASS = new DynExtDynamicClassPub();

//*******************************************
// access public method of parent class
// from outside of class
//*******************************************

arr = new Array(1, 2, 3);

// ********************************************
// access public method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );



// ********************************************
// access public method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access public method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access public method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access public property from outside
// the class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public property from outside the class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubArray = arr", arr, (EXTDCLASS.pubArray = arr, EXTDCLASS.pubArray) );


// ********************************************
// access public property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, (EXTDCLASS.testSubDPArray(arr)) );


// ********************************************
// access public property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access public property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubDPArray(arr)", arr, (EXTDCLASS.testPrivSubDPArray(arr)) );


// ********************************************
// access public property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPub();
Assert.expectEq( "*** Access public property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubDPArray(arr)", arr, (EXTDCLASS.testFinSubDPArray(arr)) );



              // displays results.
