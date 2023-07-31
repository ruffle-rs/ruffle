/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the cosine of x. The
argument is expressed in radians.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.7";
// var VERSION = "AS3";
// var TITLE   = "public native static function cos (x:Number) :Number;";


function check(param:Number):Number { return Number.cos(param); }

Assert.expectEq("Number.cos() returns a Number", "Number", getQualifiedClassName(Number.cos(1)));
Assert.expectEq("Number.cos() length is 1", 1, Number.cos.length);
Assert.expectError("Number.cos() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.cos(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.cos(undefined)", NaN, Number.cos(undefined));
Assert.expectEq("Number.cos(string)", NaN, Number.cos("string"));
Assert.expectEq("Number.cos(NaN)", NaN, Number.cos(NaN));
Assert.expectEq("Number.cos(NaN) check()", NaN, check(NaN));

// If x is +0, the result is 1.
Assert.expectEq("Number.cos(0)", 1, Number.cos(0));
Assert.expectEq("Number.cos('0')", 1, Number.cos('0'));
Assert.expectEq("Number.cos(0) check()", 1, check(0));

// If x is -0, the result is 1.
Assert.expectEq("Number.cos(-0)", 1, Number.cos(-0));
Assert.expectEq("Number.cos('-0')", 1, Number.cos('-0'));
Assert.expectEq("Number.cos(-0) check()", 1, check(-0));

// If x is +Infinity, the result is NaN.
Assert.expectEq("Number.cos(Number.POSITIVE_INFINITY)", NaN, Number.cos(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.cos(Number.POSITIVE_INFINITY) check()", NaN, check(Number.POSITIVE_INFINITY));

// If x is -Infinity, the result is NaN.
Assert.expectEq("Number.cos(Number.NEGATIVE_INFINITY)", NaN, Number.cos(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.cos(Number.NEGATIVE_INFINITY) check()", NaN, check(Number.NEGATIVE_INFINITY));


Assert.expectEq("Number.cos(null)", 1, Number.cos(null));
Assert.expectEq("Number.cos(true)", 0.5403022766113281, Number.cos(true));
Assert.expectEq("Number.cos(false)", 1, Number.cos(false));

Assert.expectEq("Number.cos(Number.PI)", -1, Number.cos(Number.PI));
Assert.expectEq("Number.cos(-Number.PI)", -1, Number.cos(-Number.PI));

var myNum:Number = 3.1415927;
Assert.expectEq("Number.cos(myNum=3.1415927)", -1, Number.cos(myNum));
Assert.expectEq("Number.cos(myNum=-3.1415927)", -1, Number.cos(-myNum));

Assert.expectEq("Number.cos(3.1415927) NumberLiteral", -1, Number.cos(3.1415927));
Assert.expectEq("Number.cos(-3.1415927) NumberLiteral", -1, Number.cos(-3.1415927));

Assert.expectEq("Number.cos(Number.MIN_VALUE)", 1, Number.cos(Number.MIN_VALUE));
Assert.expectEq("Number.cos(Number.MAX_VALUE)", -0.9999876894265599, Number.cos(Number.MAX_VALUE));


