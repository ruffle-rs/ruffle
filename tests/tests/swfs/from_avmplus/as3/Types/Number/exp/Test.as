/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the exponential function
of x (e raised to the power of x, where e is the base of the natural logarithms).
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.8";
// var VERSION = "AS3";
// var TITLE   = "public native static function exp (x:Number) :Number;";


function check(param:Number):Number { return Number.exp(param); }

Assert.expectEq("Number.exp() returns a Number", "Number", getQualifiedClassName(Number.exp(1)));
Assert.expectEq("Number.exp() length is 1", 1, Number.exp.length);
Assert.expectError("Number.exp() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.exp(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.exp(undefined)", NaN, Number.exp(undefined));
Assert.expectEq("Number.exp(string)", NaN, Number.exp("string"));
Assert.expectEq("Number.exp(NaN)", NaN, Number.exp(NaN));
Assert.expectEq("Number.exp(NaN) check()", NaN, check(NaN));

// If x is +0, the result is 1.
Assert.expectEq("Number.exp(0)", 1, Number.exp(0));
Assert.expectEq("Number.exp('0')", 1, Number.exp('0'));
Assert.expectEq("Number.exp(0) check()", 1, check(0));

// If x is -0, the result is 1.
Assert.expectEq("Number.exp(-0)", 1, Number.exp(-0));
Assert.expectEq("Number.exp('-0')", 1, Number.exp('-0'));
Assert.expectEq("Number.exp(-0) check()", 1, check(-0));

// If x is +Infinity, the result is +Infinity.
Assert.expectEq("Number.exp(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.exp(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.exp(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

// If x is -Infinity, the result is +0.
Assert.expectEq("Number.exp(Number.NEGATIVE_INFINITY)", 0, Number.exp(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.exp(Number.NEGATIVE_INFINITY) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.exp(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.exp(Number.NEGATIVE_INFINITY) check()", 0, check(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.exp(Number.NEGATIVE_INFINITY) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(Number.NEGATIVE_INFINITY));


Assert.expectEq("Number.exp(null)", 1, Number.exp(null));
Assert.expectEq("Number.exp(false)", 1, Number.exp(false));

Assert.expectEq("Number.exp(1)", Number.E, Number.exp(1));
Assert.expectEq("Number.exp(1) check()", Number.E, check(1));
Assert.expectEq("Number.exp('1')", Number.E, Number.exp('1'));
Assert.expectEq("Number.exp(true)", Number.E, Number.exp(true));

Assert.expectEq("Number.exp(Number.MIN_VALUE)", 1, Number.exp(Number.MIN_VALUE));
Assert.expectEq("Number.exp(Number.MAX_VALUE)", Number.POSITIVE_INFINITY, Number.exp(Number.MAX_VALUE));

Assert.expectEq("Number.exp(1.0e+3)", Number.POSITIVE_INFINITY, Number.exp(1.0e+3));
Assert.expectEq("Number.exp(-1.0e+3)", 0, Number.exp(-1.0e+3));


