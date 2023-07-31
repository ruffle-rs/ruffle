/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns the smallest (closest to -Infinity) Number value that is not less than x
and is equal to a mathematical integer. If x is already an integer, the result is x.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.6";
// var VERSION = "AS3";
// var TITLE   = "public native static function ceil (x:Number) :Number;";


function check(param:Number):Number { return Number.ceil(param); }

Assert.expectEq("Number.ceil() returns a int", "int", getQualifiedClassName(Number.ceil(1.125)));
Assert.expectEq("Number.ceil() length is 1", 1, Number.ceil.length);
Assert.expectError("Number.ceil() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.ceil(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.ceil(undefined)", NaN, Number.ceil(undefined));
Assert.expectEq("Number.ceil(string)", NaN, Number.ceil("string"));
Assert.expectEq("Number.ceil(NaN)", NaN, Number.ceil(NaN));
Assert.expectEq("Number.ceil(NaN) check()", NaN, check(NaN));

// If x is +0, the result is +0.
Assert.expectEq("Number.ceil(0)", 0, Number.ceil(0));
Assert.expectEq("Number.ceil(0) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(0));
Assert.expectEq("Number.ceil(0) check()", 0, check(0));
Assert.expectEq("Number.ceil(0) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));

// If x is -0, the result is -0.
Assert.expectEq("Number.ceil(-0)", -0, Number.ceil(-0));
Assert.expectEq("Number.ceil(-0) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(-0));
Assert.expectEq("Number.ceil(-0) check()", -0, check(-0));
Assert.expectEq("Number.ceil(-0) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));

// If x is +Infinity, the result is +Infinty.
Assert.expectEq("Number.ceil(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.ceil(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.ceil(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

// If x is -Infinity, the result is -Infinity.
Assert.expectEq("Number.ceil(Number.NEGATIVE_INFINITY)", Number.NEGATIVE_INFINITY, Number.ceil(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.ceil(Number.NEGATIVE_INFINITY) check()", Number.NEGATIVE_INFINITY, check(Number.NEGATIVE_INFINITY));

// If x is less than 0 but greater than -1, the result is -0.
Assert.expectEq("Number.ceil(-0.1)", Number(-0), Number.ceil(-0.1));
Assert.expectEq("Number that Number.ceil(-0.1) returns -0", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(-0.1));
Assert.expectEq("Number.ceil(-0.5)", -0, Number.ceil(-0.5));
Assert.expectEq("Number.ceil(-0.5) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(-0.5));
Assert.expectEq("Number.ceil(-0.999)", -0, Number.ceil(-0.999));
Assert.expectEq("Number.ceil(-0.999) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(-0.999));
Assert.expectEq("Number.ceil(-0.5) check()", -0, check(-0.5));
Assert.expectEq("Number.ceil(-0.5) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0.5));
Assert.expectEq("Number.ceil(-0.999) check()", -0, check(-0.999));
Assert.expectEq("Number.ceil(-0.999) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0.999));

// The value of Number.ceil(x) is the same as the value of -Number.floor(-x).
Assert.expectEq("Number.ceil(3.124) == -Number.floor(-3.124)", -Number.floor(-3.124), Number.ceil(3.124));

Assert.expectEq("Number.ceil(null)", 0, Number.ceil(null));
Assert.expectEq("Number.ceil(true)", 1, Number.ceil(true));
Assert.expectEq("Number.ceil(false)", 0, Number.ceil(false));

Assert.expectEq("Number.ceil(Number.MAX_VALUE)", Number.MAX_VALUE, Number.ceil(Number.MAX_VALUE));
Assert.expectEq("Number.ceil(Number.MAX_VALUE+1.79769313486231e+308)", Number.POSITIVE_INFINITY, Number.ceil(Number.MAX_VALUE+1.79769313486231e+308));
Assert.expectEq("Number.ceil(Number.MIN_VALUE)", 1, Number.ceil(Number.MIN_VALUE));

Assert.expectEq("Number.ceil('1')", 1, Number.ceil('1'));
Assert.expectEq("Number.ceil('0')", 0, Number.ceil('0'));

var myNum:Number = 1;
Assert.expectEq("Number.ceil(myNum=1)", 1, Number.ceil(myNum));
myNum = 0;
Assert.expectEq("Number.ceil(myNum=0)", 0, Number.ceil(myNum));
Assert.expectEq("Number.INFINITY/Number.ceil(myNum=0)", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(myNum));
myNum = -0;
Assert.expectEq("Number.ceil(myNum=-0)", -0, Number.ceil(myNum));
Assert.expectEq("Number.INFINITY/Number.ceil(myNum=-0)", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.ceil(myNum));
myNum = -1;
Assert.expectEq("Number.ceil(myNum=-1)", -1, Number.ceil(myNum));

Assert.expectEq("Number.ceil(1) NumberLiteral", 1, Number.ceil(1));
Assert.expectEq("Number.ceil(0) NumberLiteral", 0, Number.ceil(0));
Assert.expectEq("Number.ceil(-0) NumberLiteral", -0, Number.ceil(-0));
Assert.expectEq("Number.ceil(-1) NumberLiteral", -1, Number.ceil(-1));


