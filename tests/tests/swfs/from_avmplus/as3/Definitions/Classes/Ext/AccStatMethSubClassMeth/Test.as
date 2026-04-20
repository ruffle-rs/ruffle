/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "Access static method of base class from subclass";       // Provide ECMA section title or a description
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

import StaticPropertyPackage.*;

import com.adobe.test.Assert;
var obj = new AccStatMethSubClassMeth();

// ********************************************
// Access the static method via BaseClass.foo()
// ********************************************
Assert.expectEq( "*** Access the static method via base class ***", 1, 1 );
Assert.expectEq( "BaseClass.echo('hello')", "hello", BaseClass.echo("hello") );

// ********************************************
// Access the static method via sub class,
// using unadorned "foo()"
// ********************************************
Assert.expectEq( "*** Access the static method via sub class using unadorned method call ***", 1, 1 );
Assert.expectEq( "obj.callEcho('world')", "world", obj.callEcho("world") );

// ********************************************
// Access the static method via sub class,
// using "BaseClass.foo()"
// ********************************************
Assert.expectEq( "*** Access the static method via sub class using base class method call ***", 1, 1 );
Assert.expectEq( "obj.callBaseEcho('here')", "here", obj.callBaseEcho("here") );

// ********************************************
// Access the static method via sub class using
// subclass.foo() in a method
// ********************************************
// no longer legal as of d359
//Assert.expectEq( "*** Access the static method via sub class using subclass.foo() in a method ***", 1, 1 );
//Assert.expectEq( "obj.callSubEcho('there')", "there", obj.callSubEcho("there") );


              // displays results.
