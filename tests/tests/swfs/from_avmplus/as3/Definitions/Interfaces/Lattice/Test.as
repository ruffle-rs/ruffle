/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Interface Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import Lattice.*;

import com.adobe.test.Assert;
var fg1 = new ImplFG
Assert.expectEq("public on unambiguous interface method (f)", "IFuncF::f", fg1.f())
Assert.expectEq("public on unambiguous interface method (g)", "IFuncG::g", fg1.g())

var fxg1 = new ImplGxF
Assert.expectEq("public on unambiguous extended interface method (f)", "IFuncF::f", fxg1.f())
Assert.expectEq("public on unambiguous interface method (g)", "IFuncGxF::g", fxg1.g())

var hxfg1 = new ImplHxFG
Assert.expectEq("extended implementation method (f)", "IFuncF::f", hxfg1.f())
Assert.expectEq("extended implementation method (g)", "IFuncG::g", hxfg1.g())
Assert.expectEq("public on unambiguous interface method (h)", "IFuncH::h", hxfg1.h())

              // displays results.
