/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

/**
 *  All 'import' statements  should be the first
 *  in a file.
 */
import PublicClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";                // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";                 // Version of JavaScript or ECMA
// var TITLE   = "dynamic Class Extends Public Class";     // Provide ECMA section title or a description
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

var DYNEXTDCLASS = new DynExtPublicClassFin();
var arr = new Array(1, 2, 3);


// ********************************************
// access final method from a default
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from default method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testSubArray(arr)", arr, (DYNEXTDCLASS.testSubArray(arr)) );



// ********************************************
// access final method from a public
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from public method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()", arr, (DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()) );


// ********************************************
// access final method from a private
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from private method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivSubArray(arr)", arr, DYNEXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access final method from a final
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from final method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testFinSubArray(arr)", arr, (DYNEXTDCLASS.testFinSubArray(arr)) );


// ********************************************
// access final method from a public final
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from public final method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubFinSubSetArray(arr), DYNEXTDCLASS.pubFinSubGetArray()", arr, (DYNEXTDCLASS.pubFinSubSetArray(arr), DYNEXTDCLASS.pubFinSubGetArray()) );

// ********************************************
// access final method from a private final
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final method from private final method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivFinSubArray(arr)", arr, (DYNEXTDCLASS.testPrivFinSubArray(arr)) );



// ********************************************
// access final property from
// default method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testSubDPArray(arr)", arr, (DYNEXTDCLASS.testSubDPArray(arr)) );


// ********************************************
// access final property from
// public method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from public method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubSubSetDPArray(arr), DYNEXTDCLASS.pubSubGetDPArray()", arr, (DYNEXTDCLASS.pubSubSetDPArray(arr), DYNEXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access final property from
// private method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from private method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivSubDPArray(arr)", arr, (DYNEXTDCLASS.testPrivSubDPArray(arr)) );



// ********************************************
// access final property from
// final method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from final method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testFinSubDPArray(arr)", arr, (DYNEXTDCLASS.testFinSubDPArray(arr)) );

// ********************************************
// access final property from
// virtual method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from virtual method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testVirSubDPArray(arr)", arr, (DYNEXTDCLASS.testVirSubDPArray(arr)) );

// ********************************************
// access final property from
// private virtual method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassFin();
Assert.expectEq( "*** Access final property from private virtual method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivVirSubDPArray(arr)", arr, (DYNEXTDCLASS.testVirSubDPArray(arr)) );


            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
