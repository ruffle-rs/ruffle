/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import Example_9_3.*;

import com.adobe.test.Assert;
var eg = new ExampleTest();
Assert.expectEq("simple public implements", "a.T::f()", eg.doTestPublic());
Assert.expectEq("simple namespace implements", "a.g()", eg.doTestNS1());
Assert.expectEq("simple interface name implements", "b.T::f()", eg.doTestIName());
Assert.expectEq("simple namespace implements", "b.g()", eg.doTestNS2());

              // displays results.
