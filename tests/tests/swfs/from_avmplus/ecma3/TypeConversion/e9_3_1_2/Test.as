/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.3.1-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "ToNumber applied to the String type";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // A StringNumericLiteral may not use octal notation

    array[item++] = Assert.expectEq(   "Number(00)",        0,         Number("00"));
    array[item++] = Assert.expectEq(   "Number(01)",        1,         Number("01"));
    array[item++] = Assert.expectEq(   "Number(02)",        2,         Number("02"));
    array[item++] = Assert.expectEq(   "Number(03)",        3,         Number("03"));
    array[item++] = Assert.expectEq(   "Number(04)",        4,         Number("04"));
    array[item++] = Assert.expectEq(   "Number(05)",        5,         Number("05"));
    array[item++] = Assert.expectEq(   "Number(06)",        6,         Number("06"));
    array[item++] = Assert.expectEq(   "Number(07)",        7,         Number("07"));
    array[item++] = Assert.expectEq(   "Number(010)",       10,        Number("010"));
    array[item++] = Assert.expectEq(   "Number(011)",       11,        Number("011"));

    // A StringNumericLIteral may have any number of leading 0 digits

    array[item++] = Assert.expectEq(   "Number(001)",        1,         Number("001"));
    array[item++] = Assert.expectEq(   "Number(0001)",       1,         Number("0001"));

    return ( array );
}
