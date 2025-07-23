/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import StaticPropertyPackage.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Access static method of base class";       // Provide ECMA section title or a description
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

var obj:AccStatMethSubClassMethSuper = new AccStatMethSubClassMethSuper();

// ********************************************
// access static method of base class using super.foo()
// ********************************************

var thisError = "no exception thrown";
try{
    obj.getBaseDate();
} catch (e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq( "access static method of base class using 'super.foo()'",
                Utils.REFERENCEERROR+1070,
                Utils.referenceError( thisError) );
}



              // displays results.
