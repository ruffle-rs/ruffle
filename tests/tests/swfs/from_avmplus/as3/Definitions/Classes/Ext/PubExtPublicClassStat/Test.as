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

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testSubArray(arr)", arr, (PUBEXTDCLASSS.testSubArray(arr)) );


// ********************************************
// access static method from a public
// method of a sub class
//
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.pubSubSetArray(arr), PUBEXTDCLASSS.pubSubGetArray()", arr, (PUBEXTDCLASSS.pubSubSetArray(arr), PUBEXTDCLASSS.pubSubGetArray()) );


// ********************************************
// access static method from a private
// method of a sub class
//
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testPrivSubArray(arr)", arr, PUBEXTDCLASSS.testPrivSubArray(arr) );


// ********************************************
// access static method from a final
// method of a sub class
//
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "PUBEXTDCLASSS.testFinSubArray(arr)", arr, PUBEXTDCLASSS.testFinSubArray(arr) );

// ********************************************
// access static method from a static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtPublicClassStat.testStatSubArray(arr)", arr, PubExtPublicClassStat.testStatSubArray(arr) );


// ********************************************
// access static method from a public static
// method of a sub class
//
// ********************************************

Assert.expectEq( "*** Access static method from public static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtPublicClassStat.pubStatSubSetArray(arr), PubExtPublicClassStat.pubStatSubGetArray()", arr,
             (PubExtPublicClassStat.pubStatSubSetArray(arr), PubExtPublicClassStat.pubStatSubGetArray()) );


// ********************************************
// access static method from a private static
// method of a sub class
//
// ********************************************

//var PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static method from private static method of sub class ***", 1, 1 );
Assert.expectEq( "PubExtPublicClassStat.testPrivStatSubArray(arr)", arr, PubExtPublicClassStat.testPrivStatSubArray(arr) );


// ********************************************
// access static property from
// default method in sub class
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static property from default method in sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testSubDPArray(arr)", arr, PUBEXTDCLASSS.testSubDPArray(arr) );



// ********************************************
// access static property from
// public method in sub class
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.pubSubSetDPArray(arr), PUBEXTDCLASSS.pubSubGetDPArray()", arr, (PUBEXTDCLASSS.pubSubSetDPArray(arr), PUBEXTDCLASSS.pubSubGetDPArray()) );


// ********************************************
// access static property from
// private method in sub class
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static property from private method in sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testPrivSubDPArray(arr)", arr, PUBEXTDCLASSS.testPrivSubDPArray(arr) );


// ********************************************
// access static property from
// static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "PubExtPublicClassStat.testStatSubPArray(arr)", arr, PubExtPublicClassStat.testStatSubPArray(arr) );

// ********************************************
// access static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "PubExtPublicClassStat.pubStatSubSetSPArray(arr), PubExtPublicClassStat.pubStatSubGetSPArray()", arr,
             (PubExtPublicClassStat.pubStatSubSetSPArray(arr), PubExtPublicClassStat.pubStatSubGetSPArray()) );

// ********************************************
// access static property from
// private static method in sub class
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testPrivStatSubPArray(arr)", arr, PUBEXTDCLASSS.testPrivStatSubPArray(arr));



// ********************************************
// access static property from
// final method in sub class
// ********************************************

PUBEXTDCLASSS = new PubExtPublicClassStat();
Assert.expectEq( "*** Access static property from final method in sub class ***", 1, 1 );
Assert.expectEq( "PUBEXTDCLASSS.testFinSubDPArray(arr)", arr, PUBEXTDCLASSS.testFinSubDPArray(arr) );


            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
