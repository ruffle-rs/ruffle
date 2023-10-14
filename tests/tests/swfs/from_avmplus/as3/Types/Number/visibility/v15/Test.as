/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
New static methods and constants on the Number object should only be
visible to SWF16 and above content.
*/

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Number constants and static methods";
// var VERSION = "AS3";
// var TITLE   = "";



/*
If the following do not produce the following error then the constants are not visisble
and the test is considered passed.
    ReferenceError: Error #1074: Illegal write to read-only property XXX on class Number.
*/
var num_e:Number = Number.E;
var num_ln10:Number = Number.LN10;
var num_ln2:Number = Number.LN2;
var num_log10e:Number = Number.LOG10E;
var num_log2e:Number = Number.LOG2E;
var num_pi:Number = Number.PI;
var num_sqrt1_2:Number = Number.SQRT1_2;
var num_sqrt2:Number = Number.SQRT2;

Number.E = 0;
Number.LN10 = 0;
Number.LN2 = 0;
Number.LOG10E = 0;
Number.LOG2E = 0;
Number.PI = 0;
Number.SQRT1_2 = 0;
Number.SQRT2 = 0;

// reset values just in case
Number.E = num_e;
Number.LN10 = num_ln10;
Number.LN2 = num_ln2;
Number.LOG10E = num_log10e;
Number.LOG2E = num_log2e;
Number.PI = num_pi;
Number.SQRT1_2 = num_sqrt1_2;
Number.SQRT2 = num_sqrt2;

Assert.expectError("Number.abs()", Utils.TYPEERROR+1006, function(){ var foo = Number.abs(1.2); });
Assert.expectError("Number.acos()", Utils.TYPEERROR+1006, function(){ var foo = Number.acos(1); });
Assert.expectError("Number.atan()", Utils.TYPEERROR+1006, function(){ var foo = Number.atan(1); });
Assert.expectError("Number.atan2()", Utils.TYPEERROR+1006, function(){ var foo = Number.atan2(0, 0); });
Assert.expectError("Number.ceil()", Utils.TYPEERROR+1006, function(){ var foo = Number.ceil(1.1); });
Assert.expectError("Number.cos()", Utils.TYPEERROR+1006, function(){ var foo = Number.cos(1); });
Assert.expectError("Number.exp()", Utils.TYPEERROR+1006, function(){ var foo = Number.exp(1); });
Assert.expectError("Number.floor()", Utils.TYPEERROR+1006, function(){ var foo = Number.floor(1); });
Assert.expectError("Number.log()", Utils.TYPEERROR+1006, function(){ var foo = Number.log(2); });
Assert.expectError("Number.max()", Utils.TYPEERROR+1006, function(){ var foo = Number.max(); });
Assert.expectError("Number.min()", Utils.TYPEERROR+1006, function(){ var foo = Number.min(); });
Assert.expectError("Number.pow()", Utils.TYPEERROR+1006, function(){ var foo = Number.pow(2, 2); });
Assert.expectError("Number.random()", Utils.TYPEERROR+1006, function(){ var foo = Number.random(); });
Assert.expectError("Number.round()", Utils.TYPEERROR+1006, function(){ var foo = Number.round(1.2); });
Assert.expectError("Number.sin()", Utils.TYPEERROR+1006, function(){ var foo = Number.sin(1); });
Assert.expectError("Number.sqrt()", Utils.TYPEERROR+1006, function(){ var foo = Number.sqrt(9); });
Assert.expectError("Number.tan()", Utils.TYPEERROR+1006, function(){ var foo = Number.tan(1); });


