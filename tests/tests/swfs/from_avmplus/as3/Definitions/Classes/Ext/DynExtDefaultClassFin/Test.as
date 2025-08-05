/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import DefaultClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";                // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                    // Version of JavaScript or ECMA
// var TITLE   = "final Class Extends Default Class";      // Provide ECMA section title or a description
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

var EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(10, 15, 20, 25, 30);

Assert.expectEq( "*** Access final method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)));


// ********************************************
// access final method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(1, 2, 3);
Assert.expectEq( "*** Access final method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubGetSetArray(arr)", arr, (EXTDCLASS.testSubGetSetArray(arr)) );



// ********************************************
// access final method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(4, 5, 6);
Assert.expectEq( "*** Access final method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access final method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(10, 50);
Assert.expectEq( "*** Access final method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access final method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(4, 5, 6);
Assert.expectEq( "*** Access final method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );



// ********************************************
// access final method from a virtual
// method of a sub class
// ********************************************

Assert.expectEq( "access 'final' method from 'virtual' method of sub class", arr,
              EXTDCLASS.testVirtSubArray(arr) );

// ********************************************
// access final property from outside
// the class
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
Assert.expectEq( "*** Access final from outside the class ***", 1, 1 );
//Assert.expectEq( "EXTDCLASS.finArray = arr", arr, (EXTDCLASS.finArray = arr, EXTDCLASS.finArray) );

// ********************************************
// access final property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
var arr = new Array(10, 20, 30);
Assert.expectEq( "*** Access final property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubGetSetDPArray(arr)", arr, (EXTDCLASS.testSubGetSetDPArray(arr)) );

// ********************************************
// access final property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
Assert.expectEq( "*** Access final property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access final property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
Assert.expectEq( "*** Access final property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubPrivDPArray(arr)", arr, (EXTDCLASS.testSubPrivDPArray(arr)) );

// ********************************************
// access final property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassFin();
Assert.expectEq( "*** Access final property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubFinDPArray(arr)", arr, (EXTDCLASS.testSubFinDPArray(arr)) );

// ********************************************
// access final property from
// virtual method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtDefaultClassFin();
Assert.expectEq( "access 'final' property from 'virtual' method of sub class", arr,
                (EXTDCLASS.testVirtSubDPArray(arr)) );

              // displays results.

