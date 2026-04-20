/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import PublicClass.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "Extend Public Class";       // Provide ECMA section title or a description
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


//*******************************************
//  access final method from
//  outside of class
//*******************************************

var EXTDCLASS = new ExtPublicClassFin();
var arr = new Array(1, 2, 3);

var thisError = "Exception not thrown!";
try{
    EXTDCLASS.finSetArray(arr);
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq( "Access final method from outside of class",
                Utils.REFERENCEERROR+1069,
                Utils.referenceError(thisError) );
}
// ********************************************
// access final method from a default
// method of a sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final method from default method of sub class",
             "1,2,3",
             EXTDCLASS.testSubGetArray(new Array(1,2,3)).toString() );

// ********************************************
// access final method from a public
// method of a sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final method from public method of sub class", arr, (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// ********************************************
// access final method from a private
// method of a sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final method from private method of sub class", arr, EXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access final method from a final
// method of a sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final method from final method of sub class", arr, EXTDCLASS.testFinSubArray(arr) );

// ********************************************
// access final method from public static
// ********************************************
thisError = "Exception not thrown!";
try{
    ExtPublicClassFin.pubStatSubGetArray();
} catch (e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq( "Access final property from outside of class",
            Utils.TYPEERROR+1006,
            Utils.typeError( thisError ) );
}

// ********************************************
// access final property from outside
// the class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
thisError = "Exception not thrown!";
try{
    EXTDCLASS.finArray;
} catch (e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq( "Access final property from outside of class",
        Utils.REFERENCEERROR+1069,
        Utils.referenceError( thisError ) );
}

// ********************************************
// access final property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final property from default method in sub class", arr, EXTDCLASS.testSubDPArray(arr));

// ********************************************
// access final property from
// public method in sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final property from public method in sub class", arr, (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );


// ********************************************
// access final property from
// final method in sub class
// ********************************************

EXTDCLASS = new ExtPublicClassFin();
Assert.expectEq( "Access final property from final method in sub class, EXTDCLASS.finSubGetDPArray()", arr, (EXTDCLASS.finSubSetDPArray(arr), EXTDCLASS.finSubGetDPArray()) );


              // displays results.
