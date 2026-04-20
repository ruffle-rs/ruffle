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
// var TITLE   = "Public Class Extends Default Class";      // Provide ECMA section title or a description
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

var EXTDCLASS = new DynExtDefaultClassPub();
var arr = new Array(10, 15, 20, 25, 30);

// *******************************************
//  access public method of parent class from
//  outside of class
// *******************************************

Assert.expectEq( "*** Access from outside of class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.setPubArray(arr), EXTDCLASS.getPubArray", arr, (EXTDCLASS.setPubArray(arr), EXTDCLASS.getPubArray()) );


// ********************************************
// access public method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );


// ********************************************
// access public method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access public method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access public method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access public method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPubFinSubArray(arr)", arr, (EXTDCLASS.testPubFinSubArray(arr)) );

// ********************************************
// access public method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivFinSubArray(arr)", arr, (EXTDCLASS.testPrivFinSubArray(arr)) );

// ********************************************
// access public method from a virtual
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from virtual method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testVirSubArray(arr)", arr, (EXTDCLASS.testVirSubArray(arr)) );

// ********************************************
// access public method from a public virtual
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from public virtual method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPubVirSubArray(arr)", arr, (EXTDCLASS.testPubVirSubArray(arr)) );

// ********************************************
// access public method from a private virtual
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public method from private virtual method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivVirSubArray(arr)", arr, (EXTDCLASS.testPrivVirSubArray(arr)) );



// ********************************************
// access public property from outside
// the class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from outside the class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubArray = arr", arr, (EXTDCLASS.pubArray = arr, EXTDCLASS.pubArray) );


// ********************************************
// access public property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, (EXTDCLASS.testSubDPArray(arr)) );


// ********************************************
// access public property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access public property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivDPArray(arr)", arr, (EXTDCLASS.testPrivDPArray(arr)) );


// ********************************************
// access public property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinDPArray(arr)", arr, (EXTDCLASS.testFinDPArray(arr)) );

// ********************************************
// access public property from
// public final method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from public final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPubFinDPArray(arr)", arr, (EXTDCLASS.testPubFinDPArray(arr)) );

// ********************************************
// access public property from
// private final method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from private final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivFinDPArray(arr)", arr, (EXTDCLASS.testPrivFinDPArray(arr)) );

// ********************************************
// access public property from
// virtual method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testVirDPArray(arr)", arr, (EXTDCLASS.testVirDPArray(arr)) );

// ********************************************
// access public property from
// public virtual method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from public final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPubVirDPArray(arr)", arr, (EXTDCLASS.testPubVirDPArray(arr)) );

// ********************************************
// access public property from
// private virtual method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassPub();
Assert.expectEq( "*** Access public property from private final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivVirDPArray(arr)", arr, (EXTDCLASS.testPrivVirDPArray(arr)) );


              // displays results.
