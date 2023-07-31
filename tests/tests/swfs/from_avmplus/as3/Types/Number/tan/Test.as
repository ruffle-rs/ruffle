/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the tangent of x. The
argument is expressed in radians.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.18";
// var VERSION = "AS3";
// var TITLE   = "public native static function tan (x:Number) :Number;";


function check(param:Number):Number { return Number.tan(param); }

Assert.expectEq("Number.tan() returns a Number", "Number", getQualifiedClassName(Number.tan(1)));
Assert.expectEq("Number.tan() length is 1", 1, Number.tan.length);
Assert.expectError("Number.tan() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.tan(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.tan(undefined)", NaN, Number.tan(undefined));
Assert.expectEq("Number.tan(string)", NaN, Number.tan("string"));
Assert.expectEq("Number.tan(NaN)", NaN, Number.tan(NaN));
Assert.expectEq("Number.tan(NaN) check()", NaN, check(NaN));

// If x is +0, the result is +0.
var zero:Number = 0;
Assert.expectEq("Number.tan(zero=0)", 0, Number.tan(zero));
Assert.expectEq("Number.tan(0) NumberLiteral", 0, Number.tan(0));
Assert.expectEq("Number.tan(0) sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.tan(0));
Assert.expectEq("Number.tan(0) check()", 0, check(0));
Assert.expectEq("Number.tan(0) check() sign check", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));
Assert.expectEq("Number.tan(null)", 0, Number.tan(null));
Assert.expectEq("Number.tan(false)", 0, Number.tan(false));

// If x is -0, the result is -0.
var neg_zero:Number = -0;
Assert.expectEq("Number.tan(neg_zero)", -0, Number.tan(neg_zero));
Assert.expectEq("Number.tan(neg_zero) sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.tan(neg_zero));
Assert.expectEq("Number.tan(-0) NumberLiteral", -0, Number.tan(-0));
Assert.expectEq("Number.tan(-0) NumberLiteral sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.tan(-0));
Assert.expectEq("Number.tan(-0) check()", -0, check(-0));
Assert.expectEq("Number.tan(-0) check() sign check", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));

// If x is +Infinity or -Infinity, the result is NaN.
Assert.expectEq("Number.tan(Number.POSITIVE_INFINITY)", NaN, Number.tan(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.tan(Number.NEGATIVE_INFINITY)", NaN, Number.tan(Number.NEGATIVE_INFINITY));
Assert.expectEq("Number.tan(Number.POSITIVE_INFINITY) check()", NaN, check(Number.POSITIVE_INFINITY));
Assert.expectEq("Number.tan(Number.NEGATIVE_INFINITY) check()", NaN, check(Number.NEGATIVE_INFINITY));


Assert.expectEq("Number.tan(Number.PI/4)", 1, Number.tan(Number.PI/4));
Assert.expectEq("Number.tan(3*Number.PI/4)", -1, Number.tan(3*Number.PI/4));
Assert.expectEq("Number.tan(Number.PI)", -Number.sin(Number.PI), Number.tan(Number.PI));


