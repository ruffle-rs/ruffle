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


var EXTDCLASS = new ExtDefaultClassPub();

//*******************************************
// public Methods and public properties
//
// call a public Method of an object that
// inherited it from it's parent class
//*******************************************

arr = new Array(1, 2, 3);
date = new Date(0);
func = new Function();
num = new Number();
obj = new Object();
str = new String("test");
//sim = new Simple();
//math = new Math();

Assert.expectEq( "*** Public Methods and Public properites ***", 1, 1 );
Assert.expectEq( "EXTDCLASS.setPubArray(arr), EXTDCLASS.pubArray", arr, (EXTDCLASS.setPubArray(arr), EXTDCLASS.pubArray) );
Assert.expectEq( "EXTDCLASS.setPubBoolean(true), EXTDCLASS.pubBoolean", true, (EXTDCLASS.setPubBoolean(true), EXTDCLASS.pubBoolean) );
//Assert.expectEq( "EXTDCLASS.setPubDate(date), EXTDCLASS.pubDate", date, (EXTDCLASS.setPubDate(date), EXTDCLASS.pubDate) );
Assert.expectEq( "EXTDCLASS.setPubFunction(func), EXTDCLASS.pubFunction", func, (EXTDCLASS.setPubFunction(func), EXTDCLASS.pubFunction) );
//Assert.expectEq( "EXTDCLASS.setPubMath(math), EXTDCLASS.pubMath", math, (EXTDCLASS.setPubMath(math), EXTDCLASS.pubMath) );
Assert.expectEq( "EXTDCLASS.setPubNumber(num), EXTDCLASS.pubNumber", num, (EXTDCLASS.setPubNumber(num), EXTDCLASS.pubNumber) );
Assert.expectEq( "EXTDCLASS.setPubObject(obj), EXTDCLASS.pubObject", obj, (EXTDCLASS.setPubObject(obj), EXTDCLASS.pubObject) );
Assert.expectEq( "EXTDCLASS.setPubString(str), EXTDCLASS.pubString", str, (EXTDCLASS.setPubString(str), EXTDCLASS.pubString) );
//Assert.expectEq( "EXTDCLASS.setPubSimple(sim), EXTDCLASS.pubSimple", sim, (EXTDCLASS.setPubSimple(sim), EXTDCLASS.pubSimple) );


// ********************************************
// access public method from a default
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.testSubArray(arr)", arr, EXTDCLASS.testSubArray(arr) );


// ********************************************
// access public method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );


// ********************************************
// access public method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access public property from outside
// the class
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.pubArray = arr", arr, (EXTDCLASS.pubArray = arr, EXTDCLASS.pubArray) );


// ********************************************
// access public property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.testSubDPArray(arr)", arr, EXTDCLASS.testSubDPArray(arr) );


// ********************************************
// access public property from
// public method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access public property from
// private method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultClassPub();
Assert.expectEq( "EXTDCLASS.testPrivSubPArray(arr)", arr, EXTDCLASS.testPrivSubPArray(arr) );

              // displays results.
