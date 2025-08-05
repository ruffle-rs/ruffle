/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import InterfaceAsType.*;

import com.adobe.test.Assert;
var eg = new TypeTest();
Assert.expectEq("class as interface, call via interface", "x.A::a()", eg.doCallXViaA());
Assert.expectEq("class as interface, call via interface", "x.B::b()", eg.doCallXViaB());
Assert.expectEq("class passed as interface, call via args", "x.A::a(),x.B::b()", eg.doCallXViaArgs());
Assert.expectEq("class returned as interface, call via result", "x.A::a(),x.B::b()", eg.doCallXViaReturn());

Assert.expectEq("class as extended interface, call via interface", "y.A::a()", eg.doCallYViaA());
Assert.expectEq("class as extended interface, call via interface", "y.B::b()", eg.doCallYViaB());
Assert.expectEq("class as extended interface, call via extended interface", "y.A::a(),y.B::b()", eg.doCallYViaC());
Assert.expectEq("class passed as extended interface, call via args", "y.A::a(),y.B::b()", eg.doCallYViaArgs());
Assert.expectEq("class passed as interface, call via extended args", "y.A::a(),y.B::b()", eg.doCallYViaArgC());
Assert.expectEq("class returned as extended interface, call via result", "y.A::a(),y.B::b()", eg.doCallYViaReturn());
Assert.expectEq("class returned as interface, call via extended result", "y.A::a(),y.B::b()", eg.doCallYViaReturnC());

              // displays results.
