/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the arc tangent of the
quotient y/x of the arguments y and x, where the signs of y and x are used to
determine the quadrant of the result. Note that it is intentional and traditional
for the two-argument arc tangent function that the argument named y be first and
the argument named x be second. The result is expressed in radians and ranges
from -PI to +PI.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.5";
// var VERSION = "AS3";
// var TITLE   = "public native static function atan2 (y:Number, x:Number):Number;";


function check(param1:Number, param2:Number):Number { return Number.atan2(param1, param2); }

Assert.expectEq("Number.atan2() returns a Number", "Number", getQualifiedClassName(Number.atan2(1,0.5)));
Assert.expectEq("Number.atan2() length is 2", 2, Number.atan2.length);
Assert.expectError("Number.atan2() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.atan2(); });
Assert.expectError("Number.atan2(0) with one args", Utils.ARGUMENTERROR+1063,  function(){ Number.atan2(0); });

// If either x or y is NaN, the result is NaN.
Assert.expectEq("Number.atan2(0, undefined)", NaN, Number.atan2(0, undefined));
Assert.expectEq("Number.atan2(undefined, 0)", NaN, Number.atan2(undefined, 0));
Assert.expectEq("Number.atan2(0, string)", NaN, Number.atan2(0, "string"));
Assert.expectEq("Number.atan2(string, 0)", NaN, Number.atan2("string", 0));
Assert.expectEq("Number.atan2(1, NaN)", NaN, Number.atan2(1, NaN));
Assert.expectEq("Number.atan2(NaN, 1)", NaN, Number.atan2(NaN, 1));
Assert.expectEq("Number.atan2(1, NaN) check()", NaN, check(1, NaN));
Assert.expectEq("Number.atan2(NaN, 1) check()", NaN, check(NaN, 1));

// If y>0 and x is +0, the result is an implementation-dependent approximation to +PI/2.
Assert.expectEq("Number.atan2(1, 0)", Number.PI/2, Number.atan2(1, 0));
Assert.expectEq("Number.atan2(1, 0) check", Number.PI/2, check(1, 0));
Assert.expectEq("Number.atan2('1', '0')", Number.PI/2, Number.atan2('1', '0'));

// If y>0 and x is -0, the result is an implementation-dependent approximation to +PI/2.
Assert.expectEq("Number.atan2(1, -0)", Number.PI/2, Number.atan2(1, -0));
Assert.expectEq("Number.atan2(1, -0) check", Number.PI/2, check(1, -0));
Assert.expectEq("Number.atan2('1', '-0')", Number.PI/2, Number.atan2('1', '-0'));

// If y is +0 and x>0, the result is +0.
Assert.expectEq("Number.atan2(0, 1)", 0, Number.atan2(0, 1));
Assert.expectEq("Number.atan2(0, 1) check via Infinity", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.atan2(0, 1));
Assert.expectEq("Number.atan2(0, 1) check()", 0, check(0, 1));
Assert.expectEq("Number.atan2(0, 1) check() check via Infinity", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0, 1));

// If y is +0 and x is +0, the result is +0.
Assert.expectEq("Number.atan2(0, 0)", 0, Number.atan2(0, 0));
Assert.expectEq("Number.atan2(0, 0) check via Infinity", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.atan2(0, 0));
Assert.expectEq("Number.atan2(0, 0) check()", 0, check(0, 0));
Assert.expectEq("Number.atan2(0, 0) check() check via Infinity", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0, 0));

// If y is +0 and x is -0, the result is an implementation-dependent approximation to +PI.
Assert.expectEq("Number.atan2(0, -0)", Number.PI, Number.atan2(0, -0));
Assert.expectEq("Number.atan2(0, -0) check()", Number.PI, check(0, -0));

// If y is +0 and x<0, the result is an implementation-dependent approximation to +PI.
Assert.expectEq("Number.atan2(0, -0.1)", Number.PI, Number.atan2(0, -0.1));
Assert.expectEq("Number.atan2(0, -0.1) check()", Number.PI, check(0, -0.1));

// If y is -0 and x>0, the result is -0.
Assert.expectEq("Number.atan2(-0, 0.1)", -0, Number.atan2(-0, 0.1));
Assert.expectEq("Number.atan2(-0, 0.1) check via Infinity", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.atan2(-0, 0.1));
Assert.expectEq("Number.atan2(-0, 0.1) check()", -0, check(-0, 0.1));
Assert.expectEq("Number.atan2(-0, 0.1) check() check via Infinity", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0, 0.1));

// If y is -0 and x is +0, the result is -0.
Assert.expectEq("Number.atan2(-0, 0)", -0, Number.atan2(-0, 0));
Assert.expectEq("Number.atan2(-0, 0) check via Infinity", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.atan2(-0, 0));
Assert.expectEq("Number.atan2(-0, 0) check()", -0, check(-0, 0));
Assert.expectEq("Number.atan2(-0, 0) check() check via Infinity", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0, 0));

// If y is -0 and x is -0, the result is an implementation-dependent approximation to -PI.
Assert.expectEq("Number.atan2(-0, -0)", -Number.PI, Number.atan2(-0, -0));
Assert.expectEq("Number.atan2(-0, -0) cehck()", -Number.PI, check(-0, -0));

// If y is -0 and x<0, the result is an implementation-dependent approximation to -PI.
Assert.expectEq("Number.atan2(-0, -0.1)", -Number.PI, Number.atan2(-0, -0.1));
Assert.expectEq("Number.atan2(-0, -0.1) check()", -Number.PI, check(-0, -0.1));

// If y<0 and x is +0, the result is an implementation-dependent approximation to -PI/2.
Assert.expectEq("Number.atan2(-0.1, 0)", -Number.PI/2, Number.atan2(-0.1, 0));
Assert.expectEq("Number.atan2(-0.1, 0) check()", -Number.PI/2, check(-0.1, 0));

// If y<0 and x is -0, the result is an implementation-dependent approximation to -PI/2.
Assert.expectEq("Number.atan2(-0.1, -0)", -Number.PI/2, Number.atan2(-0.1, -0));
Assert.expectEq("Number.atan2(-0.1, -0) check()", -Number.PI/2, check(-0.1, -0));

// If y>0 and y is finite and x is +Infinity, the result is +0.
Assert.expectEq("Number.atan2(0.1, Number.POSITIVE_INFINITY)", 0, Number.atan2(0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(0.1, Number.POSITIVE_INFINITY) check via Infinity",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/Number.atan2(0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(0.1, Number.POSITIVE_INFINITY) check()", 0, check(0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(0.1, Number.POSITIVE_INFINITY) check() check via Infinity",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(0.1, Number.POSITIVE_INFINITY));


// If y>0 and y is finite and x is -Infinity, the result if an implementation-dependent approximation to +PI.
Assert.expectEq("Number.atan2(0.1, Number.NEGATIVE_INFINITY)", Number.PI, Number.atan2(0.1, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.atan2(0.1, Number.NEGATIVE_INFINITY) check()", Number.PI, check(0.1, Number.NEGATIVE_INFINITY));

// If y<0 and y is finite and x is +Infinity, the result is -0.
Assert.expectEq("Number.atan2(-0.1, Number.POSITIVE_INFINITY)", -0, Number.atan2(0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(-0.1, Number.POSITIVE_INFINITY) check via Infinity",
            Number.NEGATIVE_INFINITY,
            Number.POSITIVE_INFINITY/Number.atan2(-0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(-0.1, Number.POSITIVE_INFINITY) check()", -0, check(0.1, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(-0.1, Number.POSITIVE_INFINITY) check() check via Infinity",
            Number.NEGATIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(-0.1, Number.POSITIVE_INFINITY));

// If y<0 and y is finite and x is -Infinity, the result is an implementation-dependent approximation to -PI.
Assert.expectEq("Number.atan2(-0.1, Number.NEGATIVE_INFINITY)", -Number.PI, Number.atan2(-0.1, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.atan2(-0.1, Number.NEGATIVE_INFINITY) check()", -Number.PI, check(-0.1, Number.NEGATIVE_INFINITY));

// If y is +Infinity and x is finite, the result is an implementation-dependent approximation to +PI/2.
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, 0)", Number.PI/2, Number.atan2(Number.POSITIVE_INFINITY, 0));
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, 0) check()", Number.PI/2, check(Number.POSITIVE_INFINITY, 0));

// If y is -Infinity and x is finite, the result is an implementation-dependent approximation to -PI/2.
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, 0)", -Number.PI/2, Number.atan2(Number.NEGATIVE_INFINITY, 0));
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, 0) check()", -Number.PI/2, check(Number.NEGATIVE_INFINITY, 0));

// If y is +Infinity and x is +Infinity, the result is an implementation-dependent approximation to +PI/4.
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY)", Number.PI/4, Number.atan2(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY) check()", Number.PI/4, check(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY));

// If y is +Infinity and x is -Infinity, the result is an implementation-dependent approximation to +3PI/4.
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)", 3*Number.PI/4, Number.atan2(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.atan2(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY) check()", 3*Number.PI/4, check(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY));

// If y is -Infinity and x is +Infinity, the result is an implementation-dependent approximation to -PI/4.
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY)", -Number.PI/4, Number.atan2(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY) check()", -Number.PI/4, check(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY));

// If y is -Infinity and x is -Infinity, the result is an implementation-dependent approximation to -3PI/4.
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY)", -3*Number.PI/4, Number.atan2(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.atan2(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY) check()", -3*Number.PI/4, check(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY));


