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
// var TITLE   = "Access static property of base class";       // Provide ECMA section title or a description
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

var obj:AccStatPropViaSubClassWIntermediate = new AccStatPropViaSubClassWIntermediate();

// ********************************************
// Try to access a static property of the base
// class via a reference to the subclass that
// extends an intermediary class
// ********************************************
var thisError = "no exception thrown";
try{
    var str = obj.string;
} catch (e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq( "access static property of base class using sub and intermediate",
                Utils.REFERENCEERROR+1069,
                Utils.referenceError( thisError) );
}


// ********************************************
// Try to access a static property in an instance
// method using an intermeidate base class
// ( C -> B -> A, in C.foo( return B.x))
// ********************************************
Assert.expectEq("*** access static property of base class using  sub and intermediate ***", 1, 1);
Assert.expectEq("obj.getString()", "baseclass", obj.getString());


              // displays results.
