/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import ImplementMultipleInterfaces.*;

import com.adobe.test.Assert;
var eg = new ImplementTest();
Assert.expectEq("single implements", "x1.A::a()", eg.doTestX1());
Assert.expectEq("double implements", "x2.A::a(),x2.B::b()", eg.doTestX2());
Assert.expectEq("triple implements", "x3.A::a(),x3.B::b(),x3.C::c()", eg.doTestX3());
Assert.expectEq("quadruple implements", "x4.A::a(),x4.B::b(),x4.C::c(),x4.D::d()", eg.doTestX4());

Assert.expectEq("extends single, single implements", "x1.A::a(),y1.B::b()", eg.doTestY1());
Assert.expectEq("extends double, double implements", "x1.A::a(),y2.B::b(),y2.C::c()", eg.doTestY2());
Assert.expectEq("extends double, double implements", "x2.A::a(),x2.B::b(),y3.C::c(),y3.D::d()", eg.doTestY3());
Assert.expectEq("extends triple, single implements", "x3.A::a(),x3.B::b(),x3.C::c(),y4.D::d()", eg.doTestY4());

              // displays results.
