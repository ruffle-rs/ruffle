/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// imports must be first statments
import DefaultClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "public extend Default Class";       // Provide ECMA section title or a description
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

//*******************************************
// access public static method of parent
// class from outside of class
//*******************************************

Assert.expectEq( "*** Public Static Methods and Public Static properites ***", 1, 1 );
//Assert.expectEq( "PubExtDefaultClassPubStat.setStatArray(arr), PubExtDefaultClassPubStat.statArray", arr,
//             (PubExtDefaultClassPubStat.setStatArray(arr), PubExtDefaultClassPubStat.statArray) );


// ********************************************
// access public static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.subSetArray(arr), EXTDCLASS.subGetArray()", arr, EXTDCLASS.testSubGetSetArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from final method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubArray(arr)", arr, EXTDCLASS.testFinSubArray(arr) );

// ********************************************
// access public static method from a static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.testStatSubArray(arr)", arr, PubExtDefaultClassPubStat.testStatSubArray(arr));

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.pubStatSubSetArray(arr), PubExtDefaultClassPubStat.pubStatSubGetArray()", arr,
             (PubExtDefaultClassPubStat.pubStatSubSetArray(arr), PubExtDefaultClassPubStat.pubStatSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a private static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.testPrivStatSubArray(arr)", arr, PubExtDefaultClassPubStat.testPrivStatSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// default method in sub class
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.subSetDPArray(arr), EXTDCLASS.subGetDPArray()", arr, EXTDCLASS.testSubGetSetDPArray(arr) );

// <TODO>  fill in the rest of the cases here


// ********************************************
// access public static property from
// public method in sub class
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// private method in sub class
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.privSubSetDPArray(arr), EXTDCLASS.privSubGetDPArray()", arr, EXTDCLASS.testPrivSubDPArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// final method in sub class
// ********************************************

EXTDCLASS = new PubExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.finSubSetDPArray(arr), EXTDCLASS.finSubGetDPArray()", arr, EXTDCLASS.testFinSubDPArray(arr) );

// ********************************************
// access public static property from
// static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.statSubSetSPArray(arr), PubExtDefaultClassPubStat.statSubGetSPArray()", arr,PubExtDefaultClassPubStat.testStatSubSPArray(arr) );

// ********************************************
// access public static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.pubStatSubSetSPArray(arr), PubExtDefaultClassPubStat.pubStatSubGetSPArray()", arr,
             (PubExtDefaultClassPubStat.pubStatSubSetSPArray(arr), PubExtDefaultClassPubStat.pubStatSubGetSPArray()) );

// ********************************************
// access public static property from
// private static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "PubExtDefaultClassPubStat.testPrivStatSubPArray(arr)", arr, PubExtDefaultClassPubStat.testPrivStatSubPArray(arr));

// <TODO>  fill in the rest of the cases here

              // displays results.
