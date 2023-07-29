/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "ToNumber";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // special cases here

    array[item++] = Assert.expectEq(    "Number()",                      0,              Number() );
    //array[item++] = Assert.expectEq(    "Number(eval('var x'))",         Number.NaN,     Number(eval("var x")) );
    var x;
    array[item++] = Assert.expectEq(    "Number(var x)",         Number.NaN,     Number(x) );
    array[item++] = Assert.expectEq(    "Number(void 0)",                Number.NaN,     Number(void 0) );
    array[item++] = Assert.expectEq(    "Number(null)",                  0,              Number(null) );
    array[item++] = Assert.expectEq(    "Number(true)",                  1,              Number(true) );
    array[item++] = Assert.expectEq(    "Number(false)",                 0,              Number(false) );
    array[item++] = Assert.expectEq(    "Number(0)",                     0,              Number(0) );
    array[item++] = Assert.expectEq(    "Number(-0)",                    -0,             Number(-0) );
    array[item++] = Assert.expectEq(    "Number(1)",                     1,              Number(1) );
    array[item++] = Assert.expectEq(    "Number(-1)",                    -1,             Number(-1) );
    array[item++] = Assert.expectEq(    "Number(Number.MAX_VALUE)",      1.7976931348623157e308, Number(Number.MAX_VALUE) );
    array[item++] = Assert.expectEq(    "Number(Number.MIN_VALUE)",      5e-324,         Number(Number.MIN_VALUE) );

    array[item++] = Assert.expectEq(    "Number(Number.NaN)",                Number.NaN,                 Number(Number.NaN) );
    array[item++] = Assert.expectEq(    "Number(Number.POSITIVE_INFINITY)",  Number.POSITIVE_INFINITY,   Number(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "Number(Number.NEGATIVE_INFINITY)",  Number.NEGATIVE_INFINITY,   Number(Number.NEGATIVE_INFINITY) );

    return ( array );
}
