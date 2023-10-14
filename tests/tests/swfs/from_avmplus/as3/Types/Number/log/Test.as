/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the natural logarithm of x.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.10";
// var VERSION = "AS3";
// var TITLE   = "public native static function log(x:Number):Number;";


function check(param:Number):Number { return Number.log(param); }

Assert.expectEq("Number.log() returns a Number", "Number", getQualifiedClassName(Number.log(12.345)));
Assert.expectEq("Number.log() length is 1", 1, Number.log.length);
Assert.expectError("Number.log() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.log(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.log(undefined)", NaN, Number.log(undefined));
Assert.expectEq("Number.log(string)", NaN, Number.log("string"));
Assert.expectEq("Number.log(NaN)", NaN, Number.log(NaN));
Assert.expectEq("Number.log(NaN) check()", NaN, check(NaN));

// If x is less than 0, the result is NaN.
Assert.expectEq("Number.log(-1)", NaN, Number.log(-1));
Assert.expectEq("Number.log(-1) check()", NaN, check(-1));
Assert.expectEq("Number.log(Number.NEGATIVE_INFINITY)", NaN, Number.log(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.log(Number.NEGATIVE_INFINITY) check()", NaN, check(Number.NEGATIVE_INFINITY));

// If x is +0 or -0, the result is -Infinity.
Assert.expectEq("Number.log(0)", Number.NEGATIVE_INFINITY, Number.log(0));
Assert.expectEq("Number.log(0) check()", Number.NEGATIVE_INFINITY, check(0));
Assert.expectEq("Number.log(-0)", Number.NEGATIVE_INFINITY, Number.log(-0));
Assert.expectEq("Number.log(-0) check()", Number.NEGATIVE_INFINITY, check(-0));
Assert.expectEq("Number.log(false)", Number.NEGATIVE_INFINITY, Number.log(false));

// If x is 1, the result is +0.
Assert.expectEq("Number.log(1)", 0, Number.log(1));
Assert.expectEq("Number.log(1) check()", 0, check(1));
Assert.expectEq("Number.log(1) is +0", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.log(1));
Assert.expectEq("Number.log(1) is +0 check()", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(1));
Assert.expectEq("Number.log(true)", 0, Number.log(true));

// If x is +Infinity, the result is +Infinity.
Assert.expectEq("Number.log(Number.POSITIVE_INFINITY)", Number.POSITIVE_INFINITY, Number.log(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.log(Number.POSITIVE_INFINITY) check()", Number.POSITIVE_INFINITY, check(Number.POSITIVE_INFINITY));

var myNum:Number = 1.0e-6;
Assert.expectEq("Number.log(-1.0e-6)", NaN, Number.log(-myNum));
Assert.expectEq("Number.log(1.0e-6)", -13.815510557964274, Number.log(myNum));
Assert.expectEq("Number.log(-1.0e-6) NumberLiteral", NaN, Number.log(-1.0e-6));
Assert.expectEq("Number.log(1.0e-6) NumberLiteral", -13.815510557964274, Number.log(1.0e-6));

myNum = 1;
Assert.expectEq("Number.log(-1)", NaN, Number.log(-myNum));
Assert.expectEq("Number.log(1)", 0, Number.log(myNum));
Assert.expectEq("Number.log(-1) NumberLiteral", NaN, Number.log(-1));
Assert.expectEq("Number.log(1) NumberLiteral", 0, Number.log(1));



