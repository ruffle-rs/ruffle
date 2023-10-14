/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
import com.adobe.test.Assert;

// var SECTION = "uint and int compare";
// var VERSION = "AS3";
// var TITLE   = "uint and signed int comparisions, bz:747437 we:747437"


function isIntGreaterUint(p_int:int, puint:uint):Boolean { return p_int > puint; }
function isUintGreaterInt(p_uint:uint, pint:int):Boolean { return p_uint > pint; }

var myInt:int;
var myUint:uint;

myInt = -30;
myUint = 6;
Assert.expectEq("isIntGreaterUint(-30, 6)", false, isIntGreaterUint(myInt, myUint));
Assert.expectEq("isUintGreaterInt(6, -30)", true, isUintGreaterInt(myUint, myInt));

myInt = 0xFFFFFFE2;
myUint = 0x00000006;
Assert.expectEq("isIntGreaterUint(0xFFFFFFE2, 0x00000006)", false, isIntGreaterUint(myInt, myUint));
Assert.expectEq("isUintGreaterInt(0x00000006, 0xFFFFFFE2)", true, isUintGreaterInt(myUint, myInt));

// Compare a negative integer to an unsigned integer that would be negative if interpreted as an int..., but more negative.
myInt = -1
myUint = 0xFFFFFFF0; // would be -16 if misinterpreted as an int
Assert.expectEq("isIntGreaterUint(-1, 0xFFFFFFF0)", false, isIntGreaterUint(myInt, myUint));
Assert.expectEq("isUintGreaterInt(0xFFFFFFF0, -1)", true, isUintGreaterInt(myUint, myInt));


