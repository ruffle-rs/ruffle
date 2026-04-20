/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import ExtendMultipleInterfaces.*;

import com.adobe.test.Assert;
var eg = new ExtendTest();
Assert.expectEq("implements single, extends single", "x1.A::a()", eg.doTestX1());
Assert.expectEq("implements single, extends double", "x2.A::a(),x2.B::b()", eg.doTestX2());
Assert.expectEq("implements single, extends single and add", "x3.C::c(),x3.I3::d()", eg.doTestX3());
Assert.expectEq("implements single, extends triple (extends single and add)", "x4.A::a(),x4.B::b(),x4.C::c(),x4.I3::d()", eg.doTestX4());

Assert.expectEq("implements single, extends double (extends single)", "y1.A::a(),y1.B::b()", eg.doTestY1());
Assert.expectEq("implements single, extends double (extends double)", "y2.A::a(),y2.B::b(),y2.C::c()", eg.doTestY2());
Assert.expectEq("implements single, extends double (extends double, extends single and add)", "y3.A::a(),y3.B::b(),y3.C::c(),y3.I3::d()", eg.doTestY3());
Assert.expectEq("implements single, extends double (extends double (extends single), extends single and add)", "y4.A::a(),y4.B::b(),y4.C::c(),y4.I3::d()", eg.doTestY4());

              // displays results.
