/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import DefaultProtClass.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Extend Default Class With Protected Variables/Methods";       // Provide ECMA section title or a description
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


// ********************************************
// access protected method from a default
// method of a sub class
//
// ********************************************

var arr = new Array(1,2,3);
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'default' method of sub class", arr, (EXTDCLASS.testSubGetSetArray(arr)) );


// ********************************************
// access protected method from a public
// method of a sub class
// ********************************************

arr = new Array( 4, 5, 6 );
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'public' method of sub class", arr,
             (EXTDCLASS.pubSubSetArray(arr), EXTDCLASS.pubSubGetArray()) );

// ********************************************
// access protected method from a final
// method of a sub class
// ********************************************

arr = new Array( "one", "two", "three" );
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'final' method of sub class", arr, (EXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access protected method from a public
// final method of a sub class
// ********************************************

arr = new Array( 8, "two", 9 );
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'public final' method of sub class", arr,
             (EXTDCLASS.pubFinSubSetArray(arr), EXTDCLASS.pubFinSubGetArray()) );

// ********************************************
// access protected method from a final
// private method of a sub class
// ********************************************

arr = new Array( "one", "two", "three" );
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'private final' method of sub class", arr, (EXTDCLASS.testPrivFinSubArray(arr)) );

// ********************************************
// access protected method from a private
// method of a sub class
// ********************************************

arr = new Array( 5, 6, 7 );
EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' method from 'private' method of sub class", arr, EXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access protected method from a virtual
// method of a sub class
// ********************************************

Assert.expectEq( "access 'protected' method from 'virtual' method of sub class", arr,
              EXTDCLASS.testVirtSubArray(arr) );

// ********************************************
// access protected method from a virtual
// public method of a sub class
// ********************************************

Assert.expectEq( "access 'protected' method from 'public virtual' method of sub class", arr,
             (EXTDCLASS.pubVirtSubSetArray(arr), EXTDCLASS.pubVirtSubGetArray()) );

// ********************************************
// access protected method from a virtual
// private method of a sub class
// ********************************************

Assert.expectEq( "access 'protected' method from 'private virtual' method of sub class", arr,
              EXTDCLASS.testPrivVirtSubArray(arr) );

// ********************************************
// access protected method from static
// method of sub class
// ********************************************

var thisError = "no exception thrown";
try{
    ExtDefaultProtClass.pubStatSubGetArray();
} catch (e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq( "access 'protected' method from 'static' method of the sub class",
                Utils.TYPEERROR+1006,
                Utils.typeError( thisError) );
}

// ********************************************
// access protected property from
// default method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'default' method of sub class", arr,
                (EXTDCLASS.testSubGetSetDPArray(arr)) );

// ********************************************
// access protected property from
// final method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'final' method of sub class", arr,
                (EXTDCLASS.testFinSubDPArray(arr)) );

// ********************************************
// access protected property from
// virtual method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'virtual' method of sub class", arr,
                (EXTDCLASS.testVirtSubDPArray(arr)) );

// ********************************************
// access protected property from
// public method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'public' method of sub class", arr,
                (EXTDCLASS.pubSubSetDPArray(arr), EXTDCLASS.pubSubGetDPArray()) );

// ********************************************
// access protected property from
// private method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'private' method of sub class", arr,
             (EXTDCLASS.testPrivSubDPArray(arr)) );

// ********************************************
// access protected property from
// public final method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'public final' method of sub class", arr,
             (EXTDCLASS.pubFinSubSetDPArray(arr), EXTDCLASS.pubFinSubGetDPArray()) );

// ********************************************
// access protected property from
// public virtual method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'public virtual' method of sub class", arr,
             (EXTDCLASS.pubVirtSubSetDPArray(arr), EXTDCLASS.pubVirtSubGetDPArray()) );

// ********************************************
// access protected property from
// private final method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'private final' method of sub class", arr,
             (EXTDCLASS.testPrivFinSubDPArray(arr)) );

// ********************************************
// access protected property from
// private virtual method in sub class
// ********************************************

EXTDCLASS = new ExtDefaultProtClass();
Assert.expectEq( "access 'protected' property from 'private virtual' method of sub class", arr,
             (EXTDCLASS.testPrivVirtSubDPArray(arr)) );

// ********************************************
// access protected property from
// static public method of sub class
// ********************************************

thisError = "no error thrown";
try{
    ExtDefaultProtClass.pubStatSubGetDPArray();
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq( "access protected property from static public method of sub class",
            Utils.TYPEERROR+1006,
            Utils.typeError( thisError ) );
}

              // displays results.
