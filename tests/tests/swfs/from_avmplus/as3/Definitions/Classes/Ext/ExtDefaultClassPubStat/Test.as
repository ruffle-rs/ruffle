/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "Extend Default Class";       // Provide ECMA section title or a description
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

import DefaultClass.*;

import com.adobe.test.Assert;
arr = new Array(1, 2, 3);

//*******************************************
// public static Methods and
// public static properties
//
// call a public static Method of an object that
// inherited it from it's parent class
//*******************************************

Assert.expectEq( "*** Public Static Methods and Public Static properites ***", 1, 1 );

/*
// needed for workaround in Flash MX 2004
Assert.expectEq( "ExtDefaultClassStat.setPubStatArray(arr), ExtDefaultClassPubStat.pubStatArray", arr,
             (DefaultClass.setPubStatArray(arr), DefaultClass.pubStatArray) );
*/

Assert.expectEq( "ExtDefaultClassPubStat.setPubStatArray(arr), ExtDefaultClassPubStat.getPubStatArray())", arr,
             (ExtDefaultClassPubStat.setPubStatArray(arr), ExtDefaultClassPubStat.getPubStatArray()) );


// ********************************************
// access public static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, EXTDCLASS.testSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassPubStat.testStatSubArray(arr)", arr, ExtDefaultClassPubStat.testStatSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access public static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassPubStat.pubStatSubSetArray(arr), ExtDefaultClassPubStat.pubStatSubGetArray()", arr,
             (ExtDefaultClassPubStat.pubStatSubSetArray(arr), ExtDefaultClassPubStat.pubStatSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static method from a private static
// method of a sub class
//
// ********************************************

var EXTDEFAULTCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDEFAULTCLASS.testPrivStatSubArray(arr)", arr, EXTDEFAULTCLASS.testPrivStatSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, EXTDCLASS.testSubDPArray(arr) );

// <TODO>  fill in the rest of the cases here


// ********************************************
// access public static property from
// public method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// private method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubDPArray(arr)", arr, EXTDCLASS.testPrivSubDPArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access public static property from
// static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassPubStat.testStatSubPArray(arr)", arr, ExtDefaultClassPubStat.testStatSubPArray(arr) );

// ********************************************
// access public static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassPubStat.pubStatSubSetSPArray(arr), ExtDefaultClassPubStat.pubStatSubGetSPArray()", arr,
             (ExtDefaultClassPubStat.pubStatSubSetSPArray(arr), ExtDefaultClassPubStat.pubStatSubGetSPArray()) );

// ********************************************
// access public static property from
// private static method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPubStat();
Assert.expectEq( "*** Access public static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivStatSubPArray(arr)", arr, EXTDCLASS.testPrivStatSubPArray(arr));

// <TODO>  fill in the rest of the cases here

              // displays results.
