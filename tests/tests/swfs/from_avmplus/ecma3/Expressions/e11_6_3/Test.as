/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_6_3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(     "Number.NaN + 1",     Number.NaN,     Number.NaN + 1 );
    array[item++] = Assert.expectEq(     "1 + Number.NaN",     Number.NaN,     1 + Number.NaN );

    array[item++] = Assert.expectEq(     "Number.NaN - 1",     Number.NaN,     Number.NaN - 1 );
    array[item++] = Assert.expectEq(     "1 - Number.NaN",     Number.NaN,     1 - Number.NaN );

    array[item++] = Assert.expectEq(   "Number.POSITIVE_INFINITY + Number.POSITIVE_INFINITY",  Number.POSITIVE_INFINITY,   Number.POSITIVE_INFINITY + Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(   "Number.NEGATIVE_INFINITY + Number.NEGATIVE_INFINITY",  Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY + Number.NEGATIVE_INFINITY);

    array[item++] = Assert.expectEq(   "Number.POSITIVE_INFINITY + Number.NEGATIVE_INFINITY",  Number.NaN,     Number.POSITIVE_INFINITY + Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(   "Number.NEGATIVE_INFINITY + Number.POSITIVE_INFINITY",  Number.NaN,     Number.NEGATIVE_INFINITY + Number.POSITIVE_INFINITY);

    array[item++] = Assert.expectEq(   "Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY",  Number.NaN,   Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(   "Number.NEGATIVE_INFINITY - Number.NEGATIVE_INFINITY",  Number.NaN,   Number.NEGATIVE_INFINITY - Number.NEGATIVE_INFINITY);

    array[item++] = Assert.expectEq(   "Number.POSITIVE_INFINITY - Number.NEGATIVE_INFINITY",  Number.POSITIVE_INFINITY,   Number.POSITIVE_INFINITY - Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(   "Number.NEGATIVE_INFINITY - Number.POSITIVE_INFINITY",  Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY - Number.POSITIVE_INFINITY);

    array[item++] = Assert.expectEq(   "-0 + -0",      -0,     -0 + -0 );
    array[item++] = Assert.expectEq(   "-0 - 0",       -0,     -0 - 0 );

    array[item++] = Assert.expectEq(   "0 + 0",        0,      0 + 0 );
    array[item++] = Assert.expectEq(   "0 + -0",       0,      0 + -0 );
    array[item++] = Assert.expectEq(   "0 - -0",       0,      0 - -0 );
    array[item++] = Assert.expectEq(   "0 - 0",        0,      0 - 0 );
    array[item++] = Assert.expectEq(   "-0 - -0",      0,     -0 - -0 );
    array[item++] = Assert.expectEq(   "-0 + 0",       0,     -0 + 0 );

    array[item++] = Assert.expectEq(   "Number.MAX_VALUE - Number.MAX_VALUE",      0,  Number.MAX_VALUE - Number.MAX_VALUE );
    array[item++] = Assert.expectEq(   "1/Number.MAX_VALUE - 1/Number.MAX_VALUE",  0,  1/Number.MAX_VALUE - 1/Number.MAX_VALUE );

    array[item++] = Assert.expectEq(   "Number.MIN_VALUE - Number.MIN_VALUE",      0,  Number.MIN_VALUE - Number.MIN_VALUE );

// the sum of an infinity and a finite value is equal to the infinite operand
    array[item++] = Assert.expectEq(   "Number.POSITIVE_INFINITY + 543.87",  Number.POSITIVE_INFINITY,   Number.POSITIVE_INFINITY + 543.87);
    array[item++] = Assert.expectEq(   "Number.NEGATIVE_INFINITY + 87456.093",  Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY + 87456.093);
    array[item++] = Assert.expectEq(   "95665 + Number.POSITIVE_INFINITY ",  Number.POSITIVE_INFINITY,   95665 + Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(   "32.543906 + Number.NEGATIVE_INFINITY ",  Number.NEGATIVE_INFINITY,   32.543906 + Number.NEGATIVE_INFINITY);

// the sum of a zero and a nonzero finite value is equal to the nonzero operand
    array[item++] = Assert.expectEq(   "0 + 40453.65",    40453.65,      0 + 40453.65 );
    array[item++] = Assert.expectEq(   "0 - 745.33",       -745.33,      0  - 745.33);
    array[item++] = Assert.expectEq(   "67007.5 + 0",     67007.5 ,      67007.5  + 0 );
    array[item++] = Assert.expectEq(   "2480.00 - 0",     2480.00 ,      2480.00 - 0 );

// the sum of two nonzero finite values of the same magnitude and opposite sign is +0
    array[item++] = Assert.expectEq(   "2480.00 + -2480.00",     +0 ,      2480.00 + -2480.00 );
    array[item++] = Assert.expectEq(   "-268.05 + 268.05",     +0 ,     -268.05 + 268.05 );
    
    //array[item++] = Assert.expectEq(   "Number.MAX_VALUE + 1",     "1.797693134862316e+308",  Number.MAX_VALUE + 1+""); [NA] this fails on FP for me
    //array[item++] = Assert.expectEq(   "Number.MAX_VALUE + 99.99",     "1.797693134862316e+308",  Number.MAX_VALUE + 99.99+"" ); [NA] this fails on FP for me
    array[item++] = Assert.expectEq(   "4324.43 + (-64.000503)",      4260.429497, 4324.43 + (-64.000503) );
    
    
    return ( array );
}
