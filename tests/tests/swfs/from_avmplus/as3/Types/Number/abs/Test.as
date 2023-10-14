/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns the absolute value of x; the result has the same magnitude as x but has positive sign.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.1";
// var VERSION = "AS3";
// var TITLE   = "public native static function abs (x:Number):Number;";


function check(param:Number):Number { return Number.abs(param); }

Assert.expectEq("Number.abs(3.14) returns a Number", "Number", getQualifiedClassName(Number.abs(3.14)));
Assert.expectEq("Number.abs(-0) returns a int", "int", getQualifiedClassName(Number.abs(-0)));
Assert.expectEq("Number.abs(1) returns a int", "int", getQualifiedClassName(Number.abs(1)));
Assert.expectEq("Number.abs() length is 1", 1, Number.abs.length);
Assert.expectError("Number.abs() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.abs(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.abs(undefined)", NaN, Number.abs(undefined));
Assert.expectEq("Number.abs(string)", NaN, Number.abs("string"));
Assert.expectEq("Number.abs(NaN)", NaN, Number.abs(NaN));
Assert.expectEq("Number.abs(NaN) check()", NaN, check(NaN));

// If x is -0, the result is +0.
Assert.expectEq("Number.abs(0.0)", 0, Number.abs(0.0));
Assert.expectEq("Number.POSITIVE_INFINITY/Number.abs(0.0)", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.abs(0.0));
Assert.expectEq("Number.abs(-0.0)", 0, Number.abs(-0.0));
Assert.expectEq("Number.POSITIVE_INFINITY/Number.abs(-0.0)", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.abs(-0.0));
Assert.expectEq("Number.POSITIVE_INFINITY/cehck(-0.0)", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0.0));

// If x is -Infinity, the result is +Infinity.
Assert.expectEq("Number.abs(Number.NEGATIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.abs(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.abs(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.abs(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.abs(Number.NEGATIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.abs(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

var pi = 3.14;
Assert.expectEq("Number.abs(-NumberLiteral)", pi, Number.abs(-pi));
Assert.expectEq("Number.abs(NumberLiteral)", pi, Number.abs(pi));
Assert.expectEq("Number.abs(-NumberLiteral) check()", pi, check(-pi));
Assert.expectEq("Number.abs(NumberLiteral) check()", pi, check(pi));
var pi:Number = 3.14;
Assert.expectEq("Number.abs(-typed)", pi, Number.abs(-pi));
Assert.expectEq("Number.abs(typed)", pi, Number.abs(pi));
Assert.expectEq("Number.abs(-typed) check()", pi, check(-pi));
Assert.expectEq("Number.abs(typed) check()", pi, check(pi));

Assert.expectEq("Number.abs(null)", 0, Number.abs(null));
Assert.expectEq("Number.abs(true)", 1, Number.abs(true));
Assert.expectEq("Number.abs(false)", 0, Number.abs(false));

Assert.expectEq("Number.abs('1')", 1, Number.abs('1'));
Assert.expectEq("Number.abs('0')", 0, Number.abs('0'));
Assert.expectEq("Number.NEGATIVE_INFINITY/Number.abs('0')", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.abs('0'));

Assert.expectEq("Number.abs(-Number.MIN_VALUE)", Number.MIN_VALUE, Number.abs(-Number.MIN_VALUE));
Assert.expectEq("Number.abs(-Number.MAX_VALUE)", Number.MAX_VALUE, Number.abs(-Number.MAX_VALUE));
Assert.expectEq("Number.abs(Number.MIN_VALUE)", Number.MIN_VALUE, Number.abs(Number.MIN_VALUE));
Assert.expectEq("Number.abs(Number.MAX_VALUE)", Number.MAX_VALUE, Number.abs(Number.MAX_VALUE));
Assert.expectEq("Number.abs(-Number.MIN_VALUE) check()", Number.MIN_VALUE, check(-Number.MIN_VALUE));
Assert.expectEq("Number.abs(-Number.MAX_VALUE) check()", Number.MAX_VALUE, check(-Number.MAX_VALUE));
Assert.expectEq("Number.abs(Number.MIN_VALUE) check()", Number.MIN_VALUE, check(Number.MIN_VALUE));
Assert.expectEq("Number.abs(Number.MAX_VALUE) check()", Number.MAX_VALUE, check(Number.MAX_VALUE));




