/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_5_2";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();
    var BUGNUMBER="111202";


function getTestCases() {
    var array = new Array();
    var item = 0;

   // if either operand is NaN, the result is NaN.

    array[item++] = Assert.expectEq(     "Number.NaN / Number.NaN",    Number.NaN,     Number.NaN / Number.NaN );
    array[item++] = Assert.expectEq(     "Number.NaN / 1",             Number.NaN,     Number.NaN / 1 );
    array[item++] = Assert.expectEq(     "1 / Number.NaN",             Number.NaN,     1 / Number.NaN );

    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / Number.NaN",    Number.NaN,     Number.POSITIVE_INFINITY / Number.NaN );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / Number.NaN",    Number.NaN,     Number.NEGATIVE_INFINITY / Number.NaN );

    // Division of an infinity by an infinity results in NaN.

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / Number.NEGATIVE_INFINITY",    Number.NaN,   Number.NEGATIVE_INFINITY / Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / Number.NEGATIVE_INFINITY",    Number.NaN,   Number.POSITIVE_INFINITY / Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / Number.POSITIVE_INFINITY",    Number.NaN,   Number.NEGATIVE_INFINITY / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / Number.POSITIVE_INFINITY",    Number.NaN,   Number.POSITIVE_INFINITY / Number.POSITIVE_INFINITY );

    // Division of an infinity by a zero results in an infinity.

    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / 0",   Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY / 0 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / 0",   Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY / 0 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / -0",  Number.NEGATIVE_INFINITY,   Number.POSITIVE_INFINITY / -0 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / -0",  Number.POSITIVE_INFINITY,   Number.NEGATIVE_INFINITY / -0 );

    // Division of an infinity by a non-zero finite value results in a signed infinity.

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / 1 ",          Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY / 1 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / -1 ",         Number.POSITIVE_INFINITY,   Number.NEGATIVE_INFINITY / -1 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / 1 ",          Number.POSITIVE_INFINITY,   Number.POSITIVE_INFINITY / 1 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / -1 ",         Number.NEGATIVE_INFINITY,   Number.POSITIVE_INFINITY / -1 );

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / Number.MAX_VALUE ",          Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY / Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY / -Number.MAX_VALUE ",         Number.POSITIVE_INFINITY,   Number.NEGATIVE_INFINITY / -Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / Number.MAX_VALUE ",          Number.POSITIVE_INFINITY,   Number.POSITIVE_INFINITY / Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY / -Number.MAX_VALUE ",         Number.NEGATIVE_INFINITY,   Number.POSITIVE_INFINITY / -Number.MAX_VALUE );

    // Division of a finite value by an infinity results in zero.

    array[item++] = Assert.expectEq(     "1 / Number.NEGATIVE_INFINITY",   -0,             1 / Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "1 / Number.POSITIVE_INFINITY",   0,              1 / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-1 / Number.POSITIVE_INFINITY",  -0,             -1 / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-1 / Number.NEGATIVE_INFINITY",  0,              -1 / Number.NEGATIVE_INFINITY );

    array[item++] = Assert.expectEq(     "Number.MAX_VALUE / Number.NEGATIVE_INFINITY",   -0,             Number.MAX_VALUE / Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.MAX_VALUE / Number.POSITIVE_INFINITY",   0,              Number.MAX_VALUE / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE / Number.POSITIVE_INFINITY",  -0,             -Number.MAX_VALUE / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE / Number.NEGATIVE_INFINITY",  0,              -Number.MAX_VALUE / Number.NEGATIVE_INFINITY );

    // Division of a zero by a zero results in NaN

    array[item++] = Assert.expectEq(     "0 / -0",                         Number.NaN,     0 / -0 );
    array[item++] = Assert.expectEq(     "-0 / 0",                         Number.NaN,     -0 / 0 );
    array[item++] = Assert.expectEq(     "-0 / -0",                        Number.NaN,     -0 / -0 );
    array[item++] = Assert.expectEq(     "0 / 0",                          Number.NaN,     0 / 0 );

    // division of zero by any other finite value results in zero

    array[item++] = Assert.expectEq(     "0 / 1",                          0,              0 / 1 );
    array[item++] = Assert.expectEq(     "0 / -1",                        -0,              0 / -1 );
    array[item++] = Assert.expectEq(     "-0 / 1",                        -0,              -0 / 1 );
    array[item++] = Assert.expectEq(     "-0 / -1",                       0,               -0 / -1 );

    // Division of a non-zero finite value by a zero results in a signed infinity.

    array[item++] = Assert.expectEq(     "1 / 0",                          Number.POSITIVE_INFINITY,   1/0 );
    array[item++] = Assert.expectEq(     "1 / -0",                         Number.NEGATIVE_INFINITY,   1/-0 );
    array[item++] = Assert.expectEq(     "-1 / 0",                         Number.NEGATIVE_INFINITY,   -1/0 );
    array[item++] = Assert.expectEq(     "-1 / -0",                        Number.POSITIVE_INFINITY,   -1/-0 );

    array[item++] = Assert.expectEq(     "0 / Number.POSITIVE_INFINITY",   0,      0 / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "0 / Number.NEGATIVE_INFINITY",   -0,     0 / Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-0 / Number.POSITIVE_INFINITY",  -0,     -0 / Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-0 / Number.NEGATIVE_INFINITY",  0,      -0 / Number.NEGATIVE_INFINITY );

    // the sign of the result is positive if both operands have the same sign
    // the sign of the result is negative if the operands have different signs

    array[item++] = Assert.expectEq(     "1 / 1",                          1,              1 / 1 );
    array[item++] = Assert.expectEq(     "1 / -1",                        -1,              1 / -1 );
    array[item++] = Assert.expectEq(     "-1 / 1",                        -1,              -1 / 1 );
    var q = -1 / -1;
    array[item++] = Assert.expectEq(     "-1 / -1",                       1,               q );

    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / Number.MAX_VALUE ",         0,   Number.MIN_VALUE / Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.MAX_VALUE / Number.MIN_VALUE ",         Number.POSITIVE_INFINITY,   Number.MAX_VALUE / Number.MIN_VALUE );
    array[item++] = Assert.expectEq(     "10.3 / 5e-324",         Number.POSITIVE_INFINITY,   10.3 / 5e-324 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 1.1 ",         4.9406564584124654e-324,   Number.MIN_VALUE / 1.1 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 1.9 ",         4.9406564584124654e-324,   Number.MIN_VALUE / 1.9 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 2.0 ",         0,   Number.MIN_VALUE / 2.0 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 3 ",         0,   Number.MIN_VALUE / 3 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 5 ",         0,   Number.MIN_VALUE / 5 );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE / 9 ",         0,   Number.MIN_VALUE / 9 );

    return ( array );
}
