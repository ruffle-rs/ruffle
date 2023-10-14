/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Class Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import bug113887.*;

import com.adobe.test.Assert;
var eg = new BugTest();
Assert.expectEq("static initialization of class directly", "yes", eg.doBasicTest());
Assert.expectEq("static initialization of class via method", "yes", eg.doFunctionTest());

              // displays results.
