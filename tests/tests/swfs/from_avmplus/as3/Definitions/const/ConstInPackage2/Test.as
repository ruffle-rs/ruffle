/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}


// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "ambinuous reference to const in package";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var num1 = 3;
var num2 = 4;

import TestPackage.*;

import com.adobe.test.Assert;
Assert.expectEq("const in package with fully qualified name", 3, TestPackage.num1 + TestPackage.num2);

