/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns the Number value that is closest to x and is equal to a mathematical
integer. If two integer Number values are equally close to x, then the result
is the Number value that is closer to +Infinity. If x is already an integer,
the result is x.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.15";
// var VERSION = "AS3";
// var TITLE   = "public native static function round (x:Number) :Number;";


function check(param:Number):Number { return Number.round(param); }

Assert.expectEq("Number.round() returns a int", "int", getQualifiedClassName(Number.round(12.345)));
Assert.expectEq("Number.round() length is 1", 1, Number.round.length);
Assert.expectError("Number.round() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.round(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.round(undefined)", NaN, Number.round(undefined));
Assert.expectEq("Number.round(string)", NaN, Number.round("string"));
Assert.expectEq("Number.round(NaN)", NaN, Number.round(NaN));
Assert.expectEq("Number.round(NaN) check()", NaN, check(NaN));

// If x is +0, the result is +0.
Assert.expectEq("Number.round(0)", 0, Number.round(0));
Assert.expectEq("Number.round(0) is +0", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(0));
Assert.expectEq("Number.round(0) check()", 0, check(0));
Assert.expectEq("Number.round(0) is +0 check()", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));
Assert.expectEq("Number.round(null)", 0, Number.round(null));
Assert.expectEq("Number.round(false)", 0, Number.round(false));

// If x is -0, the result is -0.
Assert.expectEq("Number.round(-0)", -0, Number.round(-0));
Assert.expectEq("Number.round(-0) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(-0));
Assert.expectEq("Number.round(-0) check()", -0, check(-0));
Assert.expectEq("Number.round(-0) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));

// If x is +Infinity, the result is +Infinity.
Assert.expectEq("Number.round(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.round(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.round(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

// If x is -Infinity, the result is -Infinity.
Assert.expectEq("Number.round(Number.NEGATIVE_INFINITY)", Number.NEGATIVE_INFINITY, Number.round(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.round(Number.NEGATIVE_INFINITY) check()", Number.NEGATIVE_INFINITY, check(Number.NEGATIVE_INFINITY));

// If x is greater than 0 but less than 0.5, the result is +0.
Assert.expectEq("Number.round(0.49)", 0, Number.round(0.49));
Assert.expectEq("Number.round(0.49999)", 0, Number.round(0.49999));
Assert.expectEq("Number.round(49.999e-2)", 0, Number.round(49.999e-2));
Assert.expectEq("Number.round(0.49) check()", 0, check(0.49));
Assert.expectEq("Number.round(49.999e-2) check()", 0, check(49.999e-2));
Assert.expectEq("Number.round(Number.MIN_VALUE)", 0, Number.round(Number.MIN_VALUE));

// If x is less than 0 but greater than or equal to -0.5, the result is -0.
Assert.expectEq("Number.round(-0.49)", -0, Number.round(-0.49));
Assert.expectEq("Number.round(-0.49) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(-0.49));
Assert.expectEq("Number.round(-0.49) check()", -0, check(-0.49));
Assert.expectEq("Number.round(-0.49) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0.49));
Assert.expectEq("Number.round(-0.49999)", -0, Number.round(-0.49999));
Assert.expectEq("Number.round(-0.49999) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(-0.49999));
Assert.expectEq("Number.round(-4.9999e-1)", -0, Number.round(-4.9999e-1));
Assert.expectEq("Number.round(-4.9999e-1) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(4.9999e-1));
Assert.expectEq("Number.round(-Number.MIN_VALUE)", -0, Number.round(-Number.MIN_VALUE));
Assert.expectEq("Number.round(-Number.MIN_VALUE) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(-Number.MIN_VALUE));
Assert.expectEq("Number.round(-0.5)", -0, Number.round(-0.5));
Assert.expectEq("Number.round(-0.5) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.round(-0.5));

// NOTE 1 Number.round(3.5) returns 4, but Number.round(–3.5) returns –3.
Assert.expectEq("Number.round(3.5)", 4, Number.round(3.5));
Assert.expectEq("Number.round(3.5) check()", 4, check(3.5));
Assert.expectEq("Number.round(-3.5)", -3, Number.round(-3.5));
Assert.expectEq("Number.round(-3.5) check()", -3, check(-3.5));

// NOTE 2 The value of Math.round(x) is the same as the value of Math.floor(x+0.5),
// except when x is -0 or is less than 0 but greater than or equal to -0.5; for these
// cases Math.round(x) returns -0, but Math.floor(x+0.5) returns +0.
var x = 5.26;
Assert.expectEq("x=5.26 Number.round(x) == Number.floor(x+0.5)", Number.floor(x+0.5), Number.round(x));
x = -0;
var resRound = Number.POSITIVE_INFINITY/Number.round(x);
var resFloor = Number.POSITIVE_INFINITY/Number.floor(x+0.5);
Assert.expectEq("x=-0 Number.round(x) != Number.floor(x+0.5)", true, resRound != resFloor);
x = -0.49;
var resRound = Number.POSITIVE_INFINITY/Number.round(x);
var resFloor = Number.POSITIVE_INFINITY/Number.floor(x+0.5);
Assert.expectEq("x=-0.49 Number.round(x) != Number.floor(x+0.5)", true, resRound != resFloor);



Assert.expectEq("Number.round(-5.000001e-1)", -1, Number.round(-5.000001e-1));
Assert.expectEq("Number.round(true)", 1, Number.round(true));
Assert.expectEq("Number.round(0.5)", 1, Number.round(0.5));
Assert.expectEq("Number.round(5.000001e-1)", 1, Number.round(5.000001e-1));

var myNum:Number = 3.124;
Assert.expectEq("Number.round(3.124)", 3, Number.round(myNum));
Assert.expectEq("Number.round(3.124) NumberLiteral", 3, Number.round(3.124));

Assert.expectEq("Number.round(Number.MAX_VALUE)", Number.MAX_VALUE, Number.round(Number.MAX_VALUE));



