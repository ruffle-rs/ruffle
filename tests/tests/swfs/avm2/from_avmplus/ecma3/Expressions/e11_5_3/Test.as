/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_5_3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();
    var BUGNUMBER="111202";


function getTestCases() {
    var array = new Array();
    var item = 0;

   // if either operand is NaN, the result is NaN.

    array[item++] = Assert.expectEq(     "Number.NaN % Number.NaN",    Number.NaN,     Number.NaN % Number.NaN );
    array[item++] = Assert.expectEq(     "Number.NaN % 1",             Number.NaN,     Number.NaN % 1 );
    array[item++] = Assert.expectEq(     "1 % Number.NaN",             Number.NaN,     1 % Number.NaN );

    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % Number.NaN",    Number.NaN,     Number.POSITIVE_INFINITY % Number.NaN );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % Number.NaN",    Number.NaN,     Number.NEGATIVE_INFINITY % Number.NaN );

    //  If the dividend is an infinity, or the divisor is a zero, or both, the result is NaN.
    //  dividend is an infinity

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % Number.NEGATIVE_INFINITY",    Number.NaN,   Number.NEGATIVE_INFINITY % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % Number.NEGATIVE_INFINITY",    Number.NaN,   Number.POSITIVE_INFINITY % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % Number.POSITIVE_INFINITY",    Number.NaN,   Number.NEGATIVE_INFINITY % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % Number.POSITIVE_INFINITY",    Number.NaN,   Number.POSITIVE_INFINITY % Number.POSITIVE_INFINITY );

    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % 0",   Number.NaN,     Number.POSITIVE_INFINITY % 0 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % 0",   Number.NaN,     Number.NEGATIVE_INFINITY % 0 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % -0",  Number.NaN,     Number.POSITIVE_INFINITY % -0 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % -0",  Number.NaN,     Number.NEGATIVE_INFINITY % -0 );

    array[item++] = Assert.expectEq(     "65 % 0",   Number.NaN,     65 % 0 );
    array[item++] = Assert.expectEq(     "-866.65 % 0",   Number.NaN,     -866.65 % 0 );
    array[item++] = Assert.expectEq(     "54354 % -0",  Number.NaN,     54354 % -0 );
    array[item++] = Assert.expectEq(     "876.4565 % -0",  Number.NaN,    876.4565 % -0 );

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % 1 ",  Number.NaN,     Number.NEGATIVE_INFINITY % 1 );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % -1 ", Number.NaN,     Number.NEGATIVE_INFINITY % -1 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % 1 ",  Number.NaN,     Number.POSITIVE_INFINITY % 1 );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % -1 ", Number.NaN,     Number.POSITIVE_INFINITY % -1 );

    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % Number.MAX_VALUE ",   Number.NaN,   Number.NEGATIVE_INFINITY % Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY % -Number.MAX_VALUE ",  Number.NaN,   Number.NEGATIVE_INFINITY % -Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % Number.MAX_VALUE ",   Number.NaN,   Number.POSITIVE_INFINITY % Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY % -Number.MAX_VALUE ",  Number.NaN,   Number.POSITIVE_INFINITY % -Number.MAX_VALUE );

    // divisor is 0
    array[item++] = Assert.expectEq(     "0 % -0",                         Number.NaN,     0 % -0 );
    array[item++] = Assert.expectEq(     "-0 % 0",                         Number.NaN,     -0 % 0 );
    array[item++] = Assert.expectEq(     "-0 % -0",                        Number.NaN,     -0 % -0 );
    array[item++] = Assert.expectEq(     "0 % 0",                          Number.NaN,     0 % 0 );

    array[item++] = Assert.expectEq(     "1 % 0",                          Number.NaN,   1%0 );
    array[item++] = Assert.expectEq(     "1 % -0",                         Number.NaN,   1%-0 );
    array[item++] = Assert.expectEq(     "-1 % 0",                         Number.NaN,   -1%0 );
    array[item++] = Assert.expectEq(     "-1 % -0",                        Number.NaN,   -1%-0 );

    array[item++] = Assert.expectEq(     "Number.MAX_VALUE % 0",           Number.NaN,   Number.MAX_VALUE%0 );
    array[item++] = Assert.expectEq(     "Number.MAX_VALUE % -0",          Number.NaN,   Number.MAX_VALUE%-0 );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE % 0",          Number.NaN,   -Number.MAX_VALUE%0 );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE % -0",         Number.NaN,   -Number.MAX_VALUE%-0 );

    // If the dividend is finite and the divisor is an infinity, the result equals the dividend.

    array[item++] = Assert.expectEq(     "1 % Number.NEGATIVE_INFINITY",   1,              1 % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "1 % Number.POSITIVE_INFINITY",   1,              1 % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-1 % Number.POSITIVE_INFINITY",  -1,             -1 % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-1 % Number.NEGATIVE_INFINITY",  -1,             -1 % Number.NEGATIVE_INFINITY );

    array[item++] = Assert.expectEq(     "Number.MAX_VALUE % Number.NEGATIVE_INFINITY",   Number.MAX_VALUE,    Number.MAX_VALUE % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.MAX_VALUE % Number.POSITIVE_INFINITY",   Number.MAX_VALUE,    Number.MAX_VALUE % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE % Number.POSITIVE_INFINITY",  -Number.MAX_VALUE,   -Number.MAX_VALUE % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MAX_VALUE % Number.NEGATIVE_INFINITY",  -Number.MAX_VALUE,   -Number.MAX_VALUE % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE % Number.NEGATIVE_INFINITY",   Number.MIN_VALUE,    Number.MIN_VALUE % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE % Number.POSITIVE_INFINITY",   Number.MIN_VALUE,    Number.MIN_VALUE % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MIN_VALUE % Number.POSITIVE_INFINITY",  -Number.MIN_VALUE,   -Number.MIN_VALUE % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-Number.MIN_VALUE % Number.NEGATIVE_INFINITY",  -Number.MIN_VALUE,   -Number.MIN_VALUE % Number.NEGATIVE_INFINITY );


    array[item++] = Assert.expectEq(     "0 % Number.POSITIVE_INFINITY",   0, 0 % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "0 % Number.NEGATIVE_INFINITY",   0, 0 % Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-0 % Number.POSITIVE_INFINITY",  -0,   -0 % Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "-0 % Number.NEGATIVE_INFINITY",  -0,   -0 % Number.NEGATIVE_INFINITY );

    // If the dividend is a zero and the divisor is finite, the result is the same as the dividend.

    array[item++] = Assert.expectEq(     "0 % 1",                          0,              0 % 1 );
    array[item++] = Assert.expectEq(     "0 % -1",                        0,              0 % -1 );
    array[item++] = Assert.expectEq(     "-0 % 1",                        -0,              -0 % 1 );
    array[item++] = Assert.expectEq(     "-0 % -1",                       -0,               -0 % -1 );
    
    // the sign of the result equals the sign of the dividend
    
    array[item++] = Assert.expectEq(     "10 % 3",                       1,               10 % 3 );
    array[item++] = Assert.expectEq(     "-10 % 3",                       -1,               -10 % 3 );
    array[item++] = Assert.expectEq(     "10 % -3",                       1,               10 % -3 );
    array[item++] = Assert.expectEq(     "-10 % -3",                       -1,               -10 % -3 );

//      In the remaining cases, where neither an infinity, nor a zero, nor NaN is involved, the floating-point remainder r
//      from a dividend n and a divisor d is defined by the mathematical relation r = n - (d * q) where q is an integer that
//      is negative only if n/d is negative and positive only if n/d is positive, and whose magnitude is as large as
//      possible without exceeding the magnitude of the true mathematical quotient of n and d.

    array[item++] = Assert.expectEq(     "66.6 % 25.4",                       15.799999999999997,              66.6 % 25.4 );
    array[item++] = Assert.expectEq(     "66.6 % -25.4",                       15.799999999999997,               66.6 % -25.4);
    array[item++] = Assert.expectEq(     "-66.6 % 25.4",                       -15.799999999999997,              -66.6 % 25.4 );
    array[item++] = Assert.expectEq(     "-66.6 % -25.4",                       -15.799999999999997,             -66.6 % -25.4 );
    
    // Regression for https://bugzilla.mozilla.org/show_bug.cgi?id=491084
    array[item++] = Assert.expectEq(     "null % null",        NaN, null %  null);
    array[item++] = Assert.expectEq(     "'a string' % null",  NaN, 'a string' %  null);
    array[item++] = Assert.expectEq(     "null % 'a string'",  NaN, null % 'a string');
    array[item++] = Assert.expectEq(     "Math.PI % null",     NaN, Math.PI % null);
    array[item++] = Assert.expectEq(     "null % Math.PI",     0, null % Math.PI);
    
    return ( array );
}
