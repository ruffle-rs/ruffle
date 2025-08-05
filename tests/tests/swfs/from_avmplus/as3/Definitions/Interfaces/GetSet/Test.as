/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import GetSet.*;

import com.adobe.test.Assert;
var eg = new GetSetTest();
Assert.expectEq("simple get", "x.A::get a()", eg.doGetAX());
Assert.expectEq("simple set", "x.A::set a()", eg.doSetAX());
Assert.expectEq("simple get", "x.A::get b()", eg.doGetBX());
Assert.expectEq("simple set", "x.A::set c()", eg.doSetCX());

Assert.expectEq("blend get", "y.A::get a()", eg.doGetAY());
Assert.expectEq("blend set", "y.A::set a()", eg.doSetAY());
Assert.expectEq("blend get", "y.A::get b()", eg.doGetBY());
Assert.expectEq("blend set", "y.B::set b()", eg.doSetBY());
Assert.expectEq("blend get", "y.B::get c()", eg.doGetCY());
Assert.expectEq("blend set", "y.A::set c()", eg.doSetCY());

              // displays results.
