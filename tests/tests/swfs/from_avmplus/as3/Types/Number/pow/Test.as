/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the result of raising x to the power y.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.13";
// var VERSION = "AS3";
// var TITLE   = "public native static function pow (x:Number, y:Number):Number;";


function check(param1:Number, param2:Number):Number { return Number.pow(param1, param2); }

Assert.expectEq("Number.pow() returns a Number", "Number", getQualifiedClassName(Number.pow(1.21,3.1)));
Assert.expectEq("Number.pow() length is 2", 2, Number.pow.length);
Assert.expectError("Number.pow() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.pow(); });
Assert.expectError("Number.pow() with one args", Utils.ARGUMENTERROR+1063,  function(){ Number.pow(1); });

// If y is NaN, the result is NaN.
Assert.expectEq("Number.pow(1.0, undefined)", NaN, Number.pow(1.0, undefined));
Assert.expectEq("Number.pow(1.0, string)", NaN, Number.pow(1.0, "string"));
Assert.expectEq("Number.pow(1.0, NaN)", NaN, Number.pow(1.0, NaN));
Assert.expectEq("Number.pow(1.0, NaN) check()", NaN, check(1.0, NaN));

// If y is +0, the result is 1, even if x is NaN.
Assert.expectEq("Number.pow(undefined, 0)", 1, Number.pow(undefined, 0));
Assert.expectEq("Number.pow(string, 0)", 1, Number.pow("string", 0));
Assert.expectEq("Number.pow(NaN, 0)", 1, Number.pow(NaN, 0));
Assert.expectEq("Number.pow(1.2, 0)", 1, Number.pow(1.2, 0));
Assert.expectEq("Number.pow(NaN, 0) check()", 1, check(NaN, 0));
Assert.expectEq("Number.pow(1.2, 0) check()", 1, check(1.2, 0));

// If y is -0, the result is 1, even if x is NaN.
Assert.expectEq("Number.pow(undefined, -0)", 1, Number.pow(undefined, -0));
Assert.expectEq("Number.pow(string, -0)", 1, Number.pow("string", -0));
Assert.expectEq("Number.pow(NaN, -0)", 1, Number.pow(NaN, -0));
Assert.expectEq("Number.pow(1.2, -0)", 1, Number.pow(1.2, -0));
Assert.expectEq("Number.pow(NaN, -0) check()", 1, check(NaN, -0));
Assert.expectEq("Number.pow(1.2, -0) check()", 1, check(1.2, -0));

// If x is NaN and y is nonzero, the result is NaN.
Assert.expectEq("Number.pow(undefined, 1)", NaN, Number.pow(undefined, 1));
Assert.expectEq("Number.pow(string, 1)", NaN, Number.pow("string", 1));
Assert.expectEq("Number.pow(NaN, 1)", NaN, Number.pow(NaN, 1));
Assert.expectEq("Number.pow(NaN, 1) check()", NaN, check(NaN, 1));

// If abs(x)>1 and y is +Infinity, the result is +Infinity.
Assert.expectEq("Number.pow(1.2, Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.pow(1.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.pow(-1.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(1.2, Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(1.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(-1.2, Number.POSITIVE_INFINITY));

// If abs(x)>1 and y is -Infinity, the result is +0.
Assert.expectEq("Number.pow(1.2, Number.NEGATIVE_INFINITY)", 0, Number.pow(1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(1.2, Number.NEGATIVE_INFINITY) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.NEGATIVE_INFINITY)", 0, Number.pow(-1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.NEGATIVE_INFINITY) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(-1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(1.2, Number.NEGATIVE_INFINITY) check()", 0, check(1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(1.2, Number.NEGATIVE_INFINITY) check() sign check",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.NEGATIVE_INFINITY) check()", 0, check(-1.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-1.2, Number.NEGATIVE_INFINITY) check() sign check",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(-1.2, Number.NEGATIVE_INFINITY));


// If abs(x)==1 and y is +Infinity, the result is NaN.
Assert.expectEq("Number.pow(1.0, Number.POSITIVE_INFINITY)", Number.NaN, Number.pow(1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.0, Number.POSITIVE_INFINITY)", Number.NaN, Number.pow(-1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(1.0, Number.POSITIVE_INFINITY) check()", Number.NaN, check(1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.0, Number.POSITIVE_INFINITY) check()", Number.NaN, check(-1.0, Number.POSITIVE_INFINITY));

// If abs(x)==1 and y is -Infinity, the result is NaN.
Assert.expectEq("Number.pow(1.0, Number.NEGATIVE_INFINITY)", Number.NaN, Number.pow(1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.0, Number.NEGATIVE_INFINITY)", Number.NaN, Number.pow(-1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(1.0, Number.NEGATIVE_INFINITY) check()", Number.NaN, check(1.0, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-1.0, Number.NEGATIVE_INFINITY) check()", Number.NaN, check(-1.0, Number.POSITIVE_INFINITY));

// If abs(x)<1 and y is +Infinity, the result is +0.
Assert.expectEq("Number.pow(0.2, Number.POSITIVE_INFINITY)", 0, Number.pow(0.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(0.2, Number.POSITIVE_INFINITY) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(0.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-0.2, Number.POSITIVE_INFINITY)", 0, Number.pow(-0.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(-0.2, Number.POSITIVE_INFINITY) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(-0.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(0.2, Number.POSITIVE_INFINITY) check()", 0, check(0.2, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.pow(0.2, Number.POSITIVE_INFINITY) check() sign check",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(0.2, Number.POSITIVE_INFINITY));

// If abs(x)<1 and y is -Infinity, the result is +Infinity.
Assert.expectEq("Number.pow(0.2, Number.NEGATIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.pow(0.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-0.2, Number.NEGATIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.pow(-0.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(0.2, Number.NEGATIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(0.2, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.pow(-0.2, Number.NEGATIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(-0.2, Number.NEGATIVE_INFINITY));

// If x is +Infinity and y>0, the result is +Infinity.
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, 0.1)", Number.POSITIVE_INFINITY, Number.pow(Number.POSITIVE_INFINITY, 0.1));
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, 0.1) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY, 0.1));

// If x is +Infinity and y<0, the result is +0.
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, -0.1)", 0, Number.pow(Number.POSITIVE_INFINITY, -0.1));
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, -0.1) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(Number.POSITIVE_INFINITY, -0.1));
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, -0.1) check()", 0, check(Number.POSITIVE_INFINITY, -0.1));
Assert.expectEq("Number.pow(Number.POSITIVE_INFINITY, -0.1) check() sign check",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(Number.POSITIVE_INFINITY, -0.1));

// If x is -Infinity and y>0 and y is an odd integer, the result is -Infinity.
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, 1)", Number.NEGATIVE_INFINITY, Number.pow(Number.NEGATIVE_INFINITY, 1));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, 1) check", Number.NEGATIVE_INFINITY, check(Number.NEGATIVE_INFINITY, 1));

// If x is -Infinity and y>0 and y is not an odd integer, the result is +Infinity.
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, 2)", Number.POSITIVE_INFINITY, Number.pow(Number.NEGATIVE_INFINITY, 2));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, 2) check()", Number.POSITIVE_INFINITY, check(Number.NEGATIVE_INFINITY, 2));

// If x is -Infinity and y<0 and y is an odd integer, the result is -0.
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -1)", -0, Number.pow(Number.NEGATIVE_INFINITY, -1));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -1) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(Number.NEGATIVE_INFINITY, -1));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -1) check()", -0, check(Number.NEGATIVE_INFINITY, -1));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -1) check() sign check",
            Number.NEGATIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(Number.NEGATIVE_INFINITY, -1));

// If x is -Infinity and y<0 and y is not an odd integer, the result is +0.
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -2)", 0, Number.pow(Number.NEGATIVE_INFINITY, -2));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -2) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(Number.NEGATIVE_INFINITY, -2));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -2) check()", 0, check(Number.NEGATIVE_INFINITY, -2));
Assert.expectEq("Number.pow(Number.NEGATIVE_INFINITY, -2) check() sign check",
            Number.POSITIVE_INFINITY,
            Number.POSITIVE_INFINITY/check(Number.NEGATIVE_INFINITY, -2));

// If x is +0 and y>0, the result is +0.
Assert.expectEq("Number.pow(0, 2)", 0, Number.pow(0, 2));
Assert.expectEq("Number.pow(0, 2) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(0, 2));
Assert.expectEq("Number.pow(0, 2) check()", 0, check(0, 2));
Assert.expectEq("Number.pow(0, 2) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0, 2));

// If x is +0 and y<0, the result is +Infinity.
Assert.expectEq("Number.pow(0, -2)", Number.POSITIVE_INFINITY, Number.pow(0, -2));
Assert.expectEq("Number.pow(0, -2) check()", Number.POSITIVE_INFINITY, check(0, -2));

// If x is -0 and y>0 and y is an odd integer, the result is -0.
Assert.expectEq("Number.pow(-0, 1)", -0, Number.pow(-0, 1));
Assert.expectEq("Number.pow(-0, 1) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(-0, 1));
Assert.expectEq("Number.pow(-0, 1) check()", -0, check(-0, 1));
Assert.expectEq("Number.pow(-0, 1) chec() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0, 1));

// If x is -0 and y>0 and y is not an odd integer, the result is +0.
Assert.expectEq("Number.pow(-0, 2)", 0, Number.pow(-0, 2));
Assert.expectEq("Number.pow(-0, 2) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.pow(-0, 2));
Assert.expectEq("Number.pow(-0, 2) check()", 0, check(-0, 2));
Assert.expectEq("Number.pow(-0, 2) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0, 2));

// If x is -0 and y<0 and y is an odd integer, the result is -Infinity.
Assert.expectEq("Number.pow(-0, -1)", Number.NEGATIVE_INFINITY, Number.pow(-0, -1));
Assert.expectEq("Number.pow(-0, -1) check()", Number.NEGATIVE_INFINITY, check(-0, -1));

// If x is -0 and y<0 and y is not an odd integer, the result is +Infinity.
Assert.expectEq("Number.pow(-0, -2)", Number.POSITIVE_INFINITY, Number.pow(-0, -2));
Assert.expectEq("Number.pow(-0, -2) check()", Number.POSITIVE_INFINITY, check(-0, -2));

// If x<0 and x is finite and y is finite and y is not an integer, the result is NaN.
Assert.expectEq("Number.pow(-1.125, 2.1)", NaN, Number.pow(-1.125, 2.1));
Assert.expectEq("Number.pow(-1.125, 2.1) check()", NaN, check(-1.125, 2.1));


var param1:Number = 3.14159265;
var param2:Number = 0.000001;
Assert.expectEq("Number.pow(3.14159265, 0.000001)", 1.0000011920928955, Number.pow(param1, param2));
Assert.expectEq("Number.pow(3.14159265, 0.000001) NumberLiteral", 1.0000011920928955, Number.pow(3.14159265, 0.000001));



