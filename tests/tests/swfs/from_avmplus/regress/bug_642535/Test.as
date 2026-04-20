/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import com.adobe.test.Assert;
// var SECTION = "regress_642535";
// var VERSION = "AS3";
// var TITLE   = "optimization incorrectly performs signed integer division on unsigned numbers";
// var bug = "642535";


function hide(x:Number):Number { return x; }

// We optimize certain cases of division when the result is coerced to int/uint and the rhs is a constant.

var x1 : uint = 0xffffffff;
x1 /= 3;

var x2 : uint = 0xffffffff;
x2 /= -3;

var y1 : uint = 0xffffffff;
y1 /= 32;

var y2 : uint = 0xffffffff;
y2 /= -32;

var z1 : uint = 0xffffffff;
z1 /= 0xffffffff;

var z2 : int = -1;
z2 /= 0xffffffff;

// Dividend is sufficiently large to appear negative if treated as signed.

Assert.expectEq("var x1:uint = 0xffffffff; x1 /= 3",   uint(hide(0xffffffff / 3)),   x1);
Assert.expectEq("var x2:uint = 0xffffffff; x2 /= -3",  uint(hide(0xffffffff / -3)),  x2);

Assert.expectEq("var y1:uint = 0xffffffff; y1 /= 32",   uint(hide(0xffffffff / 32)),   y1);
Assert.expectEq("var y2:uint = 0xffffffff; y2 /= -32",  uint(hide(0xffffffff / -32)),  y2);

// Divisor is sufficiently large to appear negative if treated as signed.

Assert.expectEq("var z1:uint = 0xffffffff; z1 /= 0xffffffff", uint(hide(0xffffffff / 0xffffffff)), z1);
Assert.expectEq("var z2:int = -1; z2 /= 0xffffffff", int(hide(-1 / 0xffffffff)), z2);

// Only one bit set, but number is negative.  Don't optimize this divisor as a shift!

var w1 : uint = 0xffffffff;
w1 /= -2147483648;

var w2 : int = -1;
w2 /= -2147483648;

Assert.expectEq("var w1:uint = 0xffffffff; w1 /= -2147483648", uint(hide(0xffffffff / -2147483648)), w1);
Assert.expectEq("var w2:int = -1; w2 /= -2147483648", int(hide(-1 / -2147483648)), w2);

