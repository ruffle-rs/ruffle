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
// var TITLE   = "dynamic Class Extends Public Class";         // Provide ECMA section title or a description
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

// create a new Array Object, with 3 instances.
arr = new Array(1, 2, 3);

// ********************************************
// access static method from a default
// method of a sub class
//
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testSubArray(arr)", arr, (EXTDCLASSS.testSubArray(arr)) );


// ********************************************
// access static method from a public
// method of a sub class
//
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.pubSubSetArray(arr), EXTDCLASSS.pubSubGetArray()", arr, (EXTDCLASSS.pubSubSetArray(arr), EXTDCLASSS.pubSubGetArray()) );


// ********************************************
// access static method from a private
// method of a sub class
//
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testPrivSubArray(arr)", arr, EXTDCLASSS.testPrivSubArray(arr) );


// ********************************************
// access static method from a final
// method of a sub class
//
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
//Assert.expectEq( "access static method from final method of sub class", arr, (EXTDCLASSS.testFinSubArray(arr)) );

// ********************************************
// access static method from a static
// method of a sub class
//
// ********************************************
/*
Assert.expectEq( "*** Access static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "FinExtPublicClassStat.statSubSetArray(arr), FinExtPublicClassStat.statSubGetArray()", arr,
             (FinExtPublicClassStat.statSubSetArray(arr), FinExtPublicClassStat.statSubGetArray()) );


// ********************************************
// access static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "FinExtPublicClassStat.pubStatSubSetArray(arr), FinExtPublicClassStat.pubStatSubGetArray()", arr,
             (FinExtPublicClassStat.pubStatSubSetArray(arr), FinExtPublicClassStat.pubStatSubGetArray()) );


// ********************************************
// access static method from a private static
// method of a sub class
//
// ********************************************

var EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testPrivStatSubArray(arr)", arr,
              EXTDCLASSS.testPrivStatSubArray(arr) );
*/

// ********************************************
// access static property from
// default method in sub class
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testSubDPArray(arr)", arr, (EXTDCLASSS.testSubDPArray(arr)) );



// ********************************************
// access static property from
// public method in sub class
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.pubSubSetDPArray(arr), EXTDCLASSS.pubSubGetDPArray()", arr, (EXTDCLASSS.pubSubSetDPArray(arr), EXTDCLASSS.pubSubGetDPArray()) );


// ********************************************
// access static property from
// private method in sub class
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testPrivSubDPArray(arr)", arr, (EXTDCLASSS.testPrivSubDPArray(arr)) );


// ********************************************
// access static property from
// static method in sub class
// ********************************************
/*
Assert.expectEq( "*** Access static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "FinExtPublicClassStat.statSubSetSPArray(arr), FinExtPublicClassStat.statSubGetSPArray()", arr,
             (FinExtPublicClassStat.statSubSetSPArray(arr), FinExtPublicClassStat.statSubGetSPArray()) );

// ********************************************
// access static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "FinExtPublicClassStat.pubStatSubSetSPArray(arr), FinExtPublicClassStat.pubStatSubGetSPArray()", arr,
             (FinExtPublicClassStat.pubStatSubSetSPArray(arr), FinExtPublicClassStat.pubStatSubGetSPArray()) );

// ********************************************
// access static property from
// private static method in sub class
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testPrivStatSubPArray(arr)", arr,
              EXTDCLASSS.testPrivStatSubPArray(arr));

*/

// ********************************************
// access static property from
// final method in sub class
// ********************************************

EXTDCLASSS = new DynExtPublicClassStat();
Assert.expectEq( "*** Access static property from final method in sub class ***", 1, 1 );
Assert.expectEq( "EXTDCLASSS.testFinSubDPArray(arr)", arr, (EXTDCLASSS.testFinSubDPArray(arr)) );


            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
