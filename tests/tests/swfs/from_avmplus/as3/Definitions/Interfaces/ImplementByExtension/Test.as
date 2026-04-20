/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import ImplementByExtension.*;

import com.adobe.test.Assert;
var eg = new ImplementTest();
Assert.expectEq("implements by inheritance, variety 1", "A::f(),A::f(),A::f(),A::f(),A::f(),A::f()", eg.doCallAF());
Assert.expectEq("implements by inheritance, variety 2", "A5::g(),A6::g()", eg.doCallAG());
Assert.expectEq("implements by inheritance, variety 3", "B1::f(),B2::f(),B3::f()", eg.doCallBF());
Assert.expectEq("implements by inheritance, variety 4", "B::g(),B::g(),B::g()", eg.doCallBG());
Assert.expectEq("implements by inheritance, variety 5", "A::f(),A::f(),A::f(),A::f(),A::f(),A::f(),A::f(),A::f(),A::f()", eg.doCallCF());
Assert.expectEq("implements by inheritance, variety 6", "C::g(),C::g(),C::g(),C::g(),C::g(),C::g(),C::g(),CY::g(),CY::g()", eg.doCallCG());

              // displays results.
