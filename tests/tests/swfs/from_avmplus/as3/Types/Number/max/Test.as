/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Given zero or more arguments, calls ToNumber on each of the arguments and
returns the largest of the resulting values.
*/

import avmplus.*;
import com.adobe.test.Assert;

// var SECTION = "15.8.2.11";
// var VERSION = "AS3";
// var TITLE   = "public native static function max (x:Number = NEGATIVE_INFINITY, y:Number = NEGATIVE_INFINITY, ... rest):Number;";


function checkEmpty():Number { return Number.max(); }


Assert.expectEq("Number.max() returns a Number", "Number", getQualifiedClassName(Number.max(1.25)));
Assert.expectEq("Number.max() length is 2", 2, Number.max.length);

// If no arguments are given, the result is -Infinity.
Assert.expectEq("Number.max()", Number.NEGATIVE_INFINITY, Number.max());
Assert.expectEq("Number.max() checkEmpty()", Number.NEGATIVE_INFINITY, checkEmpty());

// If any value is NaN, the result is NaN.
// undefined, "String", Number.NaN in first, second and in rest args should return Number.NaN
Assert.expectEq("Number.max(undefined, 2.1, 3.2)", NaN, Number.max(undefined, 2.1, 3.2));
Assert.expectEq("Number.max(2.1, undefined, 3.2)", NaN, Number.max(2.1, undefined, 3.2));
Assert.expectEq("Number.max(2.1, 3.2, undefined)", NaN, Number.max(2.1, 3.2, undefined));

Assert.expectEq("Number.max('string', 2.1, 3.2)", NaN, Number.max('string', 2.1, 3.2));
Assert.expectEq("Number.max(2.1, 'string', 3.2)", NaN, Number.max(2.1, 'string', 3.2));
Assert.expectEq("Number.max(2.1, 3.2, 'string')", NaN, Number.max(2.1, 3.2, 'string'));

Assert.expectEq("Number.max(Number.NaN, 2.1, 3.2)", NaN, Number.max(Number.NaN, 2.1, 3.2));
Assert.expectEq("Number.max(2.1, Number.NaN, 3.2)", NaN, Number.max(2.1, Number.NaN, 3.2));
Assert.expectEq("Number.max(2.1, 3.2, Number.NaN)", NaN, Number.max(2.1, 3.2, Number.NaN));

// The comparison of values to determine the largest value is done as in 11.8.5 except that +0 is considered to be larger than -0.
Assert.expectEq("Number.max(1, 1)", 1, Number.max(1, 1));
Assert.expectEq("Number.max(1, 0)", 1, Number.max(1, 0));
Assert.expectEq("Number.max(0, 1)", 1, Number.max(0, 1));
Assert.expectEq("Number.max(1, 1, 1)", 1, Number.max(1, 1, 1));
Assert.expectEq("Number.max(1, 0, 0)", 1, Number.max(1, 0, 0));
Assert.expectEq("Number.max(0, 1, 0)", 1, Number.max(0, 1, 0));
Assert.expectEq("Number.max(0, 0, 1)", 1, Number.max(0, 0, 1));
Assert.expectEq("Number.max(1, 1, 0)", 1, Number.max(1, 1, 0));
Assert.expectEq("Number.max(1, 0, 1)", 1, Number.max(1, 0, 1));
Assert.expectEq("Number.max(0, 1, 1)", 1, Number.max(0, 1, 1));

/*
Do the following combinations, treating 1=0 and 0=-0. This will check that handling -0 is correct
for both x and y, PLUS that the rest args are checked properly.
    1, 1
    0, 0
    1, 0
    0, 1
    1, 1, 1
    0, 0, 0
    1, 0, 0
    0, 1, 0
    0, 0, 1
    1, 1, 0
    1, 0, 1
    0, 1, 1
*/
// The comparison of values to determine the largest value is done as in 11.8.5 except that +0 is considered to be larger than -0.
function isPositive(param:Number):Boolean
{
    return Number.POSITIVE_INFINITY/param == Number.POSITIVE_INFINITY;
}
Assert.expectEq("Number.max( 0,  0)", 0, Number.max(0, 0));
Assert.expectEq("Number.max( 0,  0) check sign", true, isPositive(Number.max(0, 0)));
Assert.expectEq("Number.max(-0, -0)", -0, Number.max(-0, -0));
Assert.expectEq("Number.max(-0, -0) check sign", false, isPositive(Number.max(-0, -0)));
Assert.expectEq("Number.max( 0, -0)", 0, Number.max(0, -0));
Assert.expectEq("Number.max( 0, -0) check sign", true, isPositive(Number.max(0, -0)));
Assert.expectEq("Number.max(-0,  0)", 0, Number.max(-0, 0));
Assert.expectEq("Number.max(-0,  0) check sign", true, isPositive(Number.max(-0, 0)));
Assert.expectEq("Number.max( 0,  0,  0)", 0, Number.max(0, 0, 0));
Assert.expectEq("Number.max( 0,  0,  0) check sign", true, isPositive(Number.max(0, 0, 0)));
Assert.expectEq("Number.max(-0, -0, -0)", -0, Number.max(-0, -0, -0));
Assert.expectEq("Number.max(-0, -0, -0) check sign", false, isPositive(Number.max(-0, -0, -0)));
Assert.expectEq("Number.max( 0, -0, -0)", 0, Number.max(0, -0, -0));
Assert.expectEq("Number.max( 0, -0, -0) check sign", true, isPositive(Number.max(0, -0, -0)));
Assert.expectEq("Number.max(-0,  0, -0)", 0, Number.max(-0, 0, -0));
Assert.expectEq("Number.max(-0,  0, -0) check sign", true, isPositive(Number.max(-0, 0, -0)));
Assert.expectEq("Number.max(-0, -0,  0)", 0, Number.max(-0, -0, 0));
Assert.expectEq("Number.max(-0, -0,  0) check sign", true, isPositive(Number.max(-0, -0, 0)));
Assert.expectEq("Number.max( 0,  0, -0)", 0, Number.max(0, 0, -0));
Assert.expectEq("Number.max( 0,  0, -0) check sign", true, isPositive(Number.max(0, 0, -0)));
Assert.expectEq("Number.max( 0, -0,  0)", 0, Number.max(0, -0, 0));
Assert.expectEq("Number.max( 0, -0,  0) check sign", true, isPositive(Number.max(0, -0, 0)));
Assert.expectEq("Number.max(-0,  0,  0)", 0, Number.max(-0, 0, 0));
Assert.expectEq("Number.max(-0,  0,  0) check sign", true, isPositive(Number.max(-0, 0, 0)));


Assert.expectEq("Number.max(null, 1)", 1, Number.max(null, 1));
Assert.expectEq("Number.max(-1, null)", 0, Number.max(-1, null));
Assert.expectEq("Number.max(false, true)", 1, Number.max(false, true));


Assert.expectEq("Number.max(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.max(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY));
Assert.expectEq("Number.max(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.max(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.max(Number.MIN_VALUE, 0)", Number.MIN_VALUE, Number.max(Number.MIN_VALUE, 0));

Assert.expectEq("Number.max(Number.POSITIVE_INFINITY, Number.MAX_VALUE)", Number.POSITIVE_INFINITY, Number.max(Number.POSITIVE_INFINITY, Number.MAX_VALUE));


