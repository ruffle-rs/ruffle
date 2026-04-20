/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "extend public class";       // Provide ECMA section title or a description
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

import PublicClass.*;

import com.adobe.test.Assert;
import com.adobe.test.Utils;


// ********************************************
// access default method from a default
// method of a sub class
//
// ********************************************

var arr = new Array(1, 2, 3);

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.testSubGetSetArray(arr)", arr, EXTDCLASS.testSubGetSetArray(arr) );


// ********************************************
// access default method from a public
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// ********************************************
// access default method from a private
// method of a sub class
//
// ********************************************

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.testPrivSubArray(arr)", arr, EXTDCLASS.testPrivSubArray(arr) );


// ********************************************
// access default property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.testSubGetSetDPArray(arr)", arr, EXTDCLASS.testSubGetSetDPArray(arr) );


// ********************************************
// access default property from
// ********************************************

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );



// ********************************************
// access default property from
// private method in sub class
// ********************************************

EXTDCLASS = new ExtPublicClass();
Assert.expectEq( "EXTDCLASS.testPrivSubDPArray(arr)", arr, EXTDCLASS.testPrivSubDPArray(arr) );

var thisError = "no Exception thrown!";
try{
    ExtPublicClass.pubStatSubGetDPArray();
} catch (e){
    thisError = e.toString();
} finally {
    Assert.expectEq( "access default property from public static method of sub class",
                    Utils.TYPEERROR+1006,
                    Utils.typeError( thisError ) );
}

              // displays results.
