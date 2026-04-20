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
// var TITLE   = "dynamic Class Extends Public Class";      // Provide ECMA section title or a description
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

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static method from default method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testSubArray(arr)", arr, (DYNEXTDCLASS.testSubArray(arr)) );


// ********************************************
// access public static method from a public
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static method from public method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()", arr, (DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()) );


// ********************************************
// access public static method from a private
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static method from private method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivSubArray(arr)", arr, DYNEXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access public static method from a final
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static method from final method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testFinSubArray(arr)", arr, (DYNEXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access public static method from a static
// method of a sub class
//
// ********************************************
/*
Assert.expectEq( "*** Access public static method from static method of sub class ***", 1, 1 );
Assert.expectEq( "DynExtPublicClassPubStat.statSubSetArray(arr), DynExtPublicClassPubStat.statSubGetArray()", arr,
             (DynExtPublicClassPubStat.statSubSetArray(arr), DynExtPublicClassPubStat.statSubGetArray()) );
*/

// ********************************************
// access public static method from a public static
// method of a sub class
//
// ********************************************

//Assert.expectEq( "*** Access public static method from public static method of sub class ***", 1, 1 );
//Assert.expectEq( "DynExtPublicClassPubStat.pubStatSubSetArray(arr), DynExtPublicClassPubStat.pubStatSubGetArray()", arr,
 //            (DynExtPublicClassPubStat.pubStatSubSetArray(arr), DynExtPublicClassPubStat.pubStatSubGetArray()) );


// ********************************************
// access public static method from a private static
// method of a sub class
//
// ********************************************

var EXTDEFAULTCLASS = new DynExtPublicClassPubStat();
//Assert.expectEq( "*** Access public static method from private static method of sub class ***", 1, 1 );
//Assert.expectEq( "DynExtPublicClassPubStat.testPrivStatSubArray(arr)", arr,
//              DynExtPublicClassPubStat.testPrivStatSubArray(arr) );


// ********************************************
// access public static property from
// default method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
//Assert.expectEq( "*** Access public static property from default method in sub class ***", 1, 1 );
//Assert.expectEq( "DYNEXTDCLASS.subSetDPArray(arr), DYNEXTDCLASS.subGetDPArray()", arr, (DYNEXTDCLASS.subSetDPArray(arr), DYNEXTDCLASS.subGetDPArray()) );



// ********************************************
// access public static property from
// public method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static property from public method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubSubSetDPArray(arr), DYNEXTDCLASS.pubSubGetDPArray()", arr, (DYNEXTDCLASS.pubSubSetDPArray(arr), DYNEXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access public static property from
// private method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
//Assert.expectEq( "*** Access public static property from private method in sub class ***", 1, 1 );
//Assert.expectEq( "DYNEXTDCLASS.privSubSetDPArray(arr), DYNEXTDCLASS.privSubGetDPArray()", arr, (DYNEXTDCLASS.privSubSetDPArray(arr), DYNEXTDCLASS.privSubGetDPArray()) );


// ********************************************
// access public static property from
// final method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
//Assert.expectEq( "*** Access public static property from final method in sub class ***", 1, 1 );
//Assert.expectEq( "DYNEXTDCLASS.finSubSetDPArray(arr), DYNEXTDCLASS.finSubGetDPArray()", arr, (DYNEXTDCLASS.finSubSetDPArray(arr), DYNEXTDCLASS.finSubGetDPArray()) );

// ********************************************
// access public static property from
// static method in sub class
// ********************************************

/*
Assert.expectEq( "*** Access public static property from static method in sub class ***", 1, 1 );
Assert.expectEq( "DynExtPublicClassPubStat.statSubSetSPArray(arr), DynExtPublicClassPubStat.statSubGetSPArray()", arr,
             (DynExtPublicClassPubStat.statSubSetSPArray(arr), DynExtPublicClassPubStat.statSubGetSPArray()) );
*/

// ********************************************
// access public static property from
// public static method in sub class
// ********************************************

Assert.expectEq( "*** Access public static property from public static method in sub class ***", 1, 1 );
Assert.expectEq( "DynExtPublicClassPubStat.pubStatSubSetSPArray(arr), DynExtPublicClassPubStat.pubStatSubGetSPArray()", arr,
             (DynExtPublicClassPubStat.pubStatSubSetSPArray(arr), DynExtPublicClassPubStat.pubStatSubGetSPArray()) );

// ********************************************
// access public static property from
// private static method in sub class
// ********************************************

DYNEXTDCLASS = new DynExtPublicClassPubStat();
Assert.expectEq( "*** Access public static property from private static method in sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivStatSubPArray(arr)", arr,
              DYNEXTDCLASS.testPrivStatSubPArray(arr));



            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
