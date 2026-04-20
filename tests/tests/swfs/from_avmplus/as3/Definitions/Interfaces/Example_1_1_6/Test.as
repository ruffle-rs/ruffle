/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import Example_1_1_6.*;

import com.adobe.test.Assert;
var eg = new ExampleTest();
Assert.expectEq("simple implements, method call", "hello, world", eg.doHello());
Assert.expectEq("simple implements, method call", "goodmorning, world", eg.doGoodMorning());

              // displays results.
