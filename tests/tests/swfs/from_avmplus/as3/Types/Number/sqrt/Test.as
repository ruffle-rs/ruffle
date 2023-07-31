/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the square root of x.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.17";
// var VERSION = "AS3";
// var TITLE   = "public native static function sqrt (x:Number) :Number;";


function check(param:Number):Number { return Number.sqrt(param); }

Assert.expectEq("Number.sqrt() returns a Number", "Number", getQualifiedClassName(Number.sqrt(2)));
Assert.expectEq("Number.sqrt() length is 1", 1, Number.sqrt.length);
Assert.expectError("Number.sqrt() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.sqrt(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.sqrt(undefined)", NaN, Number.sqrt(undefined));
Assert.expectEq("Number.sqrt(string)", NaN, Number.sqrt("string"));
Assert.expectEq("Number.sqrt(NaN)", NaN, Number.sqrt(NaN));
Assert.expectEq("Number.sqrt(NaN) check()", NaN, check(NaN));

// If x is less than 0, the result is NaN.
Assert.expectEq("Number.sqrt(-0.1)", NaN, Number.sqrt(-0.1));
Assert.expectEq("Number.sqrt(-0.1) check()", NaN, check(-0.1));
Assert.expectEq("Number.sqrt(-Number.MIN_VALUE)", NaN, Number.sqrt(-Number.MIN_VALUE));
Assert.expectEq("Number.sqrt(Number.NEGATIVE_INFINITY)", NaN, Number.sqrt(Number.NEGATIVE_INFINITY));

// If x is +0, the result is +0.
var zero:Number = 0;
Assert.expectEq("Number.sqrt(zero=0)", 0, Number.sqrt(zero));
Assert.expectEq("Number.sqrt(0) NumberLiteral", 0, Number.sqrt(0));
Assert.expectEq("Number.sqrt(0) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.sqrt(0));
Assert.expectEq("Number.sqrt(0) check()", 0, check(0));
Assert.expectEq("Number.sqrt(0) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));
Assert.expectEq("Number.sqrt(null)", 0, Number.sqrt(null));
Assert.expectEq("Number.sqrt(false)", 0, Number.sqrt(false));

// If x is -0, the result is -0.
var neg_zero:Number = 0;
Assert.expectEq("Number.sqrt(neg_zero=-0)", -0, Number.sqrt(neg_zero));
Assert.expectEq("Number.sqrt(-0) NumberLiteral", -0, Number.sqrt(-0));
Assert.expectEq("Number.sqrt(-0) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.sqrt(-0));
Assert.expectEq("Number.sqrt(-0) check()", -0, check(-0));
Assert.expectEq("Number.sqrt(-0) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));


// If x is +Infinity, the result is +Infinity.
Assert.expectEq("Number.sqrt(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.sqrt(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.sqrt(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));


Assert.expectEq("Number.sqrt(true)", 1, Number.sqrt(true));
Assert.expectEq("Number.sqrt(2)", Number.SQRT2, Number.sqrt(2));
Assert.expectEq("Number.sqrt(0.5)", Number.SQRT1_2, Number.sqrt(0.5));


var i:int = 0;
var x:Number = 3;
for( i = 0; i < 20; i++ )
{
    Assert.expectEq("Number.sqrt("+x+")", Number(1.73205080756887719318).toFixed(i),  Number.sqrt(x).toFixed(i));
}
x = 2;
for( i = 0; i < 20; i++ )
{
    Assert.expectEq("Number.sqrt("+x+")", Number(1.4142135623730951455).toFixed(i),  Number.sqrt(x).toFixed(i));
}


