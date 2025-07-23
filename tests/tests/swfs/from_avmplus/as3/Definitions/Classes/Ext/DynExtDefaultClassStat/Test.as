/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import DefaultClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "static extend Default Class";       // Provide ECMA section title or a description
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

EXTDCLASS = new DynExtDefaultClassStat();
arr = new Array(1, 2, 3);

//*******************************************
// access static method of parent class
// from outside of class
//*******************************************
/*
Assert.expectEq( "Access static method of parent class from outside of class", arr,
             (EXTDCLASS.setStatArray(arr), EXTDCLASS.statArray) );

*/
// ********************************************
// access static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, (EXTDCLASS.testSubArray(arr)) );


// ********************************************
// access static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// ********************************************
// access static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access static method from a final
// method of a sub class
//
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "access static method from final method of sub class", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access static method from a static
// method of a sub class
//
// ********************************************
/*
Assert.expectEq( "*** Access static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testStatSubArray(arr)", arr,
             (EXTDCLASS.testStatSubArray(arr)) );

// ********************************************
// access static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPubStatSubArray(arr)", arr,
             (EXTDCLASS.testPubStatSubArray(arr)) );
*/
// ********************************************
// access static method from a private static
// method of a sub class
//
// ********************************************

var EXTDEFAULTCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDEFAULTCLASS.testPrivStatSubArray(arr)", arr,
              EXTDEFAULTCLASS.testPrivStatSubArray(arr) );

// ********************************************
// access static property from
// default method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testSubPArray(arr)", arr, (EXTDCLASS.testSubPArray(arr)) );


// ********************************************
// access static property from
// public method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access static property from
// private method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivSubPArray(arr)", arr, (EXTDCLASS.testPrivSubPArray(arr)) );


// ********************************************
// access static property from
// static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from static method in sub class ***", 1, 1 );
/*Assert.expectEq( "ExtDefaultClassStat.statSubSetSPArray(arr), ExtDefaultClassStat.statSubGetSPArray()", arr,
             (ExtDefaultClassStat.statSubSetSPArray(arr), ExtDefaultClassStat.statSubGetSPArray()) );
*/
// ********************************************
// access static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from public static method in sub class ***", 1, 1 );
/*Assert.expectEq( "ExtDefaultClassStat.pubStatSubSetSPArray(arr), ExtDefaultClassStat.pubStatSubGetSPArray()", arr,
             (ExtDefaultClassStat.pubStatSubSetSPArray(arr), ExtDefaultClassStat.pubStatSubGetSPArray()) );
*/
// ********************************************
// access static property from
// private static method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testPrivStatSubPArray(arr)", arr,
              EXTDCLASS.testPrivStatSubPArray(arr));

// ********************************************
// access static property from
// final method in sub class
// ********************************************

EXTDCLASS = new DynExtDefaultClassStat();
Assert.expectEq( "*** Access static property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.testFinSubPArray(arr)", arr, (EXTDCLASS.testFinSubPArray(arr)) );

              // displays results.
