/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns the greatest (closest to +Infinity) Number value that is not greater
than x and is equal to a mathematical integer. If x is already an integer, the
result is x.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.9";
// var VERSION = "AS3";
// var TITLE   = "public native static function floor (x:Number) :Number;";


function check(param:Number):Number { return Number.floor(param); }

Assert.expectEq("Number.floor() returns a int", "int", getQualifiedClassName(Number.floor(1.125)));
Assert.expectEq("Number.floor() length is 1", 1, Number.floor.length);
Assert.expectError("Number.floor() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.floor(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.floor(undefined)", NaN, Number.floor(undefined));
Assert.expectEq("Number.floor(string)", NaN, Number.floor("string"));
Assert.expectEq("Number.floor(NaN)", NaN, Number.floor(NaN));
Assert.expectEq("Number.floor(NaN)", NaN, check(NaN));

// If x is +0, the result is +0.
Assert.expectEq("Number.floor(0)", 0, Number.floor(0));
Assert.expectEq("Number.floor(0) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(0));
Assert.expectEq("Number.floor(0) check()", 0, check(0));
Assert.expectEq("Number.floor(0) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));
Assert.expectEq("Number.floor(null)", 0, Number.floor(null));
Assert.expectEq("Number.floor(false)", 0, Number.floor(false));
Assert.expectEq("Number.floor('0')", 0, Number.floor('0'));
Assert.expectEq("Number.INFINITY/Number.floor('0')", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor('0'));

// If x is -0, the result is -0.
Assert.expectEq("Number.floor(-0)", -0, Number.floor(-0));
Assert.expectEq("Number.floor(-0) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(-0));
Assert.expectEq("Number.floor(-0) check()", -0, check(-0));
Assert.expectEq("Number.floor(-0) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));

// If x is +Infinity, the result is +Infinity.
Assert.expectEq("Number.floor(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.floor(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.floor(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

// If x is -Infinity, the result is -Infinity.
Assert.expectEq("Number.floor(Number.NEGATIVE_INFINITY)", Number.NEGATIVE_INFINITY, Number.floor(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.floor(Number.NEGATIVE_INFINITY) check()", Number.NEGATIVE_INFINITY, check(Number.NEGATIVE_INFINITY));

// If x is greater than 0 but less than 1, the result is +0.
Assert.expectEq("Number.floor(Number.MIN_VALUE)", 0, Number.floor(Number.MIN_VALUE));
Assert.expectEq("Number.floor(0.5)", 0, Number.floor(0.5));
Assert.expectEq("Number.floor(0.5) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(0.5));
Assert.expectEq("Number.floor(0.999)", 0, Number.floor(0.999));
Assert.expectEq("Number.floor(0.999) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(0.999));
Assert.expectEq("Number.floor(0.5) check()", 0, check(0.5));
Assert.expectEq("Number.floor(0.5) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0.5));

// The value of Number.floor(x) is the same as the value of -Number.ceil(-x).
Assert.expectEq("Number.floor(3.124) == -Number.ceil(-3.124)", -Number.ceil(-3.124), Number.floor(3.124));



Assert.expectEq("Number.floor(true)", 1, Number.floor(true));
Assert.expectEq("Number.floor('1')", 1, Number.floor('1'));

Assert.expectEq("Number.floor(-Number.MIN_VALUE)", -1, Number.floor(-Number.MIN_VALUE));
Assert.expectEq("Number.floor(Number.MAX_VALUE)", Number.MAX_VALUE, Number.floor(Number.MAX_VALUE));

var myNum:Number = 1;
Assert.expectEq("Number.floor(myNum=1)", 1, Number.floor(myNum));
myNum = 0;
Assert.expectEq("Number.floor(myNum=0)", 0, Number.floor(myNum));
Assert.expectEq("Number.INFINITY/Number.floor(myNum=0)", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(myNum));
myNum = -0;
Assert.expectEq("Number.floor(myNum=-0)", -0, Number.floor(myNum));
Assert.expectEq("Number.INFINITY/Number.floor(myNum=-0)", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.floor(myNum));
myNum = -1;
Assert.expectEq("Number.floor(myNum=-1)", -1, Number.floor(myNum));

Assert.expectEq("Number.floor(1) NumberLiteral", 1, Number.floor(1));
Assert.expectEq("Number.floor(0) NumberLiteral", 0, Number.floor(0));
Assert.expectEq("Number.floor(-0) NumberLiteral", -0, Number.floor(-0));
Assert.expectEq("Number.floor(-1) NumberLiteral", -1, Number.floor(-1));






