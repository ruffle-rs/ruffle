/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "regress_638233";
// var VERSION = "AS3";
// var TITLE   = "constructProperty for primitive types";
// var bug = "638233";


namespace ns = "fnord"

var testnum = 1;
Assert.expectError(testnum++, "TypeError: Error #1007", function() { var a = 1; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1007", function() { var a = 3.1415; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1007", function() { var a = true; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1007", function() { var a = "string"; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1007", function() { var a = ns; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1009", function() { var a = null; new a.foo(); });
Assert.expectError(testnum++, "TypeError: Error #1010", function() { var a = undefined; new a.foo(); });

Number.prototype.foo = Number;
int.prototype.foo = int;
uint.prototype.foo = uint;

var a = 1;
Assert.expectEq(testnum++, 0, new a.foo());

var a = 3.1415;
Assert.expectEq(testnum++, 0, new a.foo());

Boolean.prototype.foo = Boolean;
var a = true;
Assert.expectEq(testnum++, false, new a.foo());

String.prototype.foo = String;
var a = "string";
Assert.expectEq(testnum++, "", new a.foo());

// ugly trick: not sure how to obtain the namespace that "new Namespace",
// so use "Number" here instead and test against that.
Namespace.prototype.foo = Number;
var a = ns;
Assert.expectEq(testnum++, 0, new a.foo());


