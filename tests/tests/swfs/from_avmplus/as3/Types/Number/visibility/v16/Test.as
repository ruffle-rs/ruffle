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


var foo;
foo = Number.abs(1.2);
foo = Number.acos(1);
foo = Number.asin(1);
foo = Number.atan(1);
foo = Number.atan2(0, 0);
foo = Number.ceil(1.1);
foo = Number.cos(1);
foo = Number.exp(1);
foo = Number.floor(1);
foo = Number.log(2);
foo = Number.max();
foo = Number.min();
foo = Number.pow(2, 2);
foo = Number.random();
foo = Number.round(1.2);
foo = Number.sin(1);
foo = Number.sqrt(9);
foo = Number.tan(1);

Assert.expectError("Number.E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.E = 0; });
Assert.expectError("Number.LN10 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LN10 = 0; });
Assert.expectError("Number.LN2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LN2 = 0; });
Assert.expectError("Number.LOG10E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LOG10E = 0; });
Assert.expectError("Number.LOG2E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LOG2E = 0; });
Assert.expectError("Number.PI - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.PI = 0; });
Assert.expectError("Number.SQRT1_2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.SQRT1_2 = 0; });
Assert.expectError("Number.SQRT2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.SQRT2 = 0; });

// Basically just pass this test, if the above didn't throw an error we are good
Assert.expectEq("New Number constants and methods are ok", true, true);

