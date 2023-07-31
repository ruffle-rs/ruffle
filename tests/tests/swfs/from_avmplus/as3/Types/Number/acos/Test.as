/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
Returns an implementation-dependent approximation to the arc cosine of x.
The result is expressed in radians and ranges from +0 to +PI.
*/

import avmplus.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "15.8.2.2";
// var VERSION = "AS3";
// var TITLE   = "public native static function acos (x:Number) :Number;";


function check(param:Number):Number { return Number.acos(param); }

Assert.expectEq("Number.acos() returns a Number", "Number", getQualifiedClassName(Number.acos(0)));
Assert.expectEq("Number.acos() length is 1", 1, Number.acos.length);
Assert.expectError("Number.acos() with no args", Utils.ARGUMENTERROR+1063,  function(){ Number.acos(); });

// If x is NaN, the result is NaN.
Assert.expectEq("Number.acos(undefined)", NaN, Number.acos(undefined));
Assert.expectEq("Number.acos(string)", NaN, Number.acos("string"));
Assert.expectEq("Number.acos(NaN)", NaN, Number.acos(NaN));
Assert.expectEq("Number.acos(NaN) check()", NaN, check(NaN));

// If x is greater than 1, the result is NaN.
Assert.expectEq("Number.acos(1.125)", NaN, Number.acos(1.125));
Assert.expectEq("Number.acos(1.125) check()", NaN, check(1.125));

// If x is less than -1, the result is NaN.
Assert.expectEq("Number.acos(-1.125)", NaN, Number.acos(-1.125));
Assert.expectEq("Number.acos(-1.125) check", NaN, check(-1.125));

// If x is exactly 1, the result is +0.
Assert.expectEq("Number.acos(1)", 0, Number.acos(1));
Assert.expectEq("Ensure that Number.acos(1) returns +0", Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY/Number.acos(1));


Assert.expectEq("Number.acos(null)", Number.PI/2.0, Number.acos(null));
Assert.expectEq("Number.acos(true)", 0, Number.acos(true));
Assert.expectEq("Number.acos(false)", Number.PI/2.0, Number.acos(false));

Assert.expectEq("Number.acos('1')", 0, Number.acos('1'));
Assert.expectEq("Number.acos('0')", Number.PI/2.0, Number.acos('0'));

var myNum:Number = 0;
Assert.expectEq("Number.acos(myNum=0)", Number.PI/2.0, Number.acos(myNum));
myNum = 1;
Assert.expectEq("Number.acos(myNum=1)", 0, Number.acos(myNum));
myNum = -1;
Assert.expectEq("Number.acos(myNum=-1)", Number.PI, Number.acos(myNum));
myNum = -0;
Assert.expectEq("Number.acos(myNum=-0)", Number.PI/2.0, Number.acos(myNum));

Assert.expectEq("Number.acos(0) NumberLiteral", Number.PI/2.0, Number.acos(0));
Assert.expectEq("Number.acos(1) NumberLiteral", 0, Number.acos(1));
Assert.expectEq("Number.acos(-1) NumberLiteral", Number.PI, Number.acos(-1));
Assert.expectEq("Number.acos(-0) NumberLiteral", Number.PI/2.0, Number.acos(-0));

Assert.expectEq("Number.acos(Number.SQRT1_2)", Number.PI/4.0, Number.acos(Number.SQRT1_2));
Assert.expectEq("Number.acos(-Number.SQRT1_2)", Number.PI/4.0*3.0, Number.acos(-Number.SQRT1_2));








