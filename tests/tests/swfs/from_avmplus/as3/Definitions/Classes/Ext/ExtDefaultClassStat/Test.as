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
// static Methods and static properties
//
// call a public Method of an object that
// inherited it from it's parent class
//*******************************************

Assert.expectEq( "*** Static Methods and Static properites ***", 1, 1 );

// needed for workaround in Flash MX 2004
/*
Assert.expectEq( "ExtDefaultClassStat.setStatArray(arr), ExtDefaultClassStat.statArray", arr,
             (DefaultClass.setStatArray(arr), DefaultClass.statArray) );

Assert.expectEq( "ExtDefaultClassStat.setStatArray(arr), ExtDefaultClassStat.statArray", arr,
             (ExtDefaultClassStat.setStatArray(arr), ExtDefaultClassStat.statArray) );
*/

// ********************************************
// access static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, EXTDCLASS.testSubArray(arr) );

// <TODO>  fill in the rest of the cases here


// ********************************************
// access static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static method from a static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassStat.testStatSubArray(arr)", arr, ExtDefaultClassStat.testStatSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassStat.pubStatSubSetArray(arr), ExtDefaultClassStat.pubStatSubGetArray()", arr,
             (ExtDefaultClassStat.pubStatSubSetArray(arr), ExtDefaultClassStat.pubStatSubGetArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static method from a private static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassStat.testPrivStatSubArray(arr)", arr, ExtDefaultClassStat.testPrivStatSubArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, EXTDCLASS.testSubDPArray(arr) );

// <TODO>  fill in the rest of the cases here


// ********************************************
// access static property from
// public method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static property from
// private method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubDPArray(arr)", arr, EXTDCLASS.testPrivSubDPArray(arr) );

// <TODO>  fill in the rest of the cases here

// ********************************************
// access static property from
// static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassStat.testStatSubPArray(arr)", arr, ExtDefaultClassStat.testStatSubPArray(arr) );

// ********************************************
// access static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "ExtDefaultClassStat.pubStatSubSetSPArray(arr), ExtDefaultClassStat.pubStatSubGetSPArray()", arr,
             (ExtDefaultClassStat.pubStatSubSetSPArray(arr), ExtDefaultClassStat.pubStatSubGetSPArray()) );

// ********************************************
// access static property from
// private static method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassStat();
Assert.expectEq( "*** Access static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivStatSubPArray(arr)", arr, EXTDCLASS.testPrivStatSubPArray(arr));

// <TODO>  fill in the rest of the cases here

              // displays results.
