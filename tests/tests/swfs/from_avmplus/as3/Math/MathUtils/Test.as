/* -*- Mode: js; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "avmplus::MathUtils";
// var VERSION = "";
// var TITLE = "Tests based on code coverage";


var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // MathUtils::convertStringToDouble(), fail when 0 digits found, but Infinity found, but string is longer than expected
    array[item++] = Assert.expectEq(   "new Number('Infinity2')", Number.NaN, new Number("Infinity2") );
    // MathUtils::convertStringToDouble(), have more than 15 digits and an exponent
    array[item++] = Assert.expectEq(   "new Number('41254121515122343e2')", 4125412151512234500, new Number("41254121515122343e2") );
    // MathUtils::convertStringToDouble(), number with 308 decimal places
    array[item++] = Assert.expectEq(   "Number with 308 decimal places", 1.1234123413412336, new Number("1.12341234134123412341243341234123413412341234124334123412341341234123412433412341234134123412341243341234123413412341234124334123412341341234123412433412341234134123412341243341234123413412341234124334123412341341234123412433412341234134123412341243341234123413412341234124334123412341341234123412433412345123") );

    // MathUtils::convertDoubleToString(), kFixedFraction write out significand
    array[item++] = Assert.expectEq(   "kFixedFraction write out significand", "0.000000120000000", new Number(0.12e-6).toFixed(15) );

    //MathUtils::convertDoubleToStringRadix
    array[item++] = Assert.expectEq(   "new Number(12.0).toString(2)",  "1100", new Number(12.0).toString(2) );
    array[item++] = Assert.expectEq(   "new Number(-1.0).toString(2)",  "-1", new Number(-1.0).toString(2) );
    array[item++] = Assert.expectEq(   "new Number(0.5).toString(2)",  "0", new Number(0.5).toString(2) );

    return ( array );
}
