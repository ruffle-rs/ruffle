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
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "Extend Dynamic Class";       // Provide ECMA section title or a description
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


arr = new Array(1, 2, 3);

// ********************************************
// access public static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );


// ********************************************
// access public static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access public static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access public static method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access public static method from a static
// method of a sub class
//
// ********************************************
/*
Assert.expectEq( "*** Access public static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtDynamicClassPubStat.statSubSetArray(arr), PubExtDynamicClassPubStat.statSubGetArray()", arr,
             (PubExtDynamicClassPubStat.statSubSetArray(arr), PubExtDynamicClassPubStat.statSubGetArray()) );


// ********************************************
// access public static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtDynamicClassPubStat.pubStatSubSetArray(arr), PubExtDynamicClassPubStat.pubStatSubGetArray()", arr,
             (PubExtDynamicClassPubStat.pubStatSubSetArray(arr), PubExtDynamicClassPubStat.pubStatSubGetArray()) );


// ********************************************
// access public static method from a private static
// method of a sub class
//
// ********************************************

var EXTDEFAULTCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDEFAULTCLASS.testPrivStatSubArray(arr)", arr,
              EXTDEFAULTCLASS.testPrivStatSubArray(arr) );

*/
// ********************************************
// access public static property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, (EXTDCLASS.testSubDPArray(arr)) );



// ********************************************
// access public static property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access public static property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubDPArray(arr)", arr, (EXTDCLASS.testPrivSubDPArray(arr)) );




// ********************************************
// access public static property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDynamicClassPubStat();
Assert.expectEq( "*** Access public static property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubPArray(arr)", arr, (EXTDCLASS.testFinSubPArray(arr)) );

              // displays results.
