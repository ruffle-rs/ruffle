/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the arc sine of x.
The result is expressed in radians and ranges from -PI/2 to +PI/2.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.3";
// var VERSION = "AS3";
// var TITLE   = "public native static function asin (x:Number) :Number;";


function check(param:Number):Number { return Number.asin(param); }

Assert.expectEq("Number.asin() returns a Number", "Number", getQualifiedClassName(Number.asin(1)));
Assert.expectEq("Number.asin() length is 1", 1, Number.asin.length);
Assert.expectError("Number.asin() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.asin(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.asin(undefined)", NaN, Number.asin(undefined));
Assert.expectEq("Number.asin(string)", NaN, Number.asin("string"));
Assert.expectEq("Number.asin(NaN)", NaN, Number.asin(NaN));

// If x is greater than 1, the result is NaN.
Assert.expectEq("Number.asin(1.125)", NaN, Number.asin(1.125));
Assert.expectEq("Number.asin(1.125) check()", NaN, check(1.125));

// If x is less than –1, the result is NaN.
Assert.expectEq("Number.asin(-1.125)", NaN, Number.asin(-1.125));
Assert.expectEq("Number.asin(-1.125) check()", NaN, check(-1.125));


// If x is +0, the result is +0.
Assert.expectEq("Number.asin(0)", 0, Number.asin(0));
Assert.expectEq("Number.asin(0) check()", 0, check(0));
Assert.expectEq("Ensure that Number.asin(+0) returns +0", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.asin(0));
Assert.expectEq("Ensure that Number.asin(+0) returns +0 check()", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/check(0));

// If x is -0, the result is -0.
Assert.expectEq("Number.asin(-0)", -0, Number.asin(-0));
Assert.expectEq("Number.asin(-0) check()", -0, check(-0));
Assert.expectEq("Ensure that Number.asin(-0) returns -0", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/Number.asin(-0));
Assert.expectEq("Ensure that Number.asin(-0) returns -0 check()", Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY/check(-0));


Assert.expectEq("Number.asin(null)", 0, Number.asin(null));
Assert.expectEq("Number.asin(true)", Number.PI/2, Number.asin(true));
Assert.expectEq("Number.asin(false)", 0, Number.asin(false));

Assert.expectEq("Number.asin('1')", Number.PI/2, Number.asin('1'));
Assert.expectEq("Number.asin('0')", 0, Number.asin('0'));

var myNum:Number = 1;
Assert.expectEq("Number.asin(myNum=1)", Number.PI/2.0, Number.asin(myNum));
myNum = 0;
Assert.expectEq("Number.asin(myNum=0)", 0, Number.asin(myNum));
myNum = -0;
Assert.expectEq("Number.asin(myNum=-0)", -0, Number.asin(myNum));
myNum = -1;
Assert.expectEq("Number.asin(myNum=-1)", -Number.PI/2.0, Number.asin(myNum));

Assert.expectEq("Number.asin(1) NumberLiteral", Number.PI/2.0, Number.asin(1));
Assert.expectEq("Number.asin(0) NumberLiteral", 0, Number.asin(0));
Assert.expectEq("Number.asin(-0) NumberLiteral", -0, Number.asin(-0));
Assert.expectEq("Number.asin(-1) NumberLiteral", -Number.PI/2.0, Number.asin(-1));

Assert.expectEq("Number.asin(Number.SQRT1_2)", 0.7853981256484985, Number.asin(Number.SQRT1_2));
Assert.expectEq("Number.asin(-Number.SQRT1_2)", -0.7853981256484985, Number.asin(-Number.SQRT1_2));




