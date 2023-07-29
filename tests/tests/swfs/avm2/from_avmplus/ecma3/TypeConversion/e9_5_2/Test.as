/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.5-2";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function ToInt32( n ) {
    n = Number( n );
    var sign = ( n < 0 ) ? -1 : 1;

    if ( Math.abs( n ) == 0 || Math.abs( n ) == Number.POSITIVE_INFINITY) {
        return 0;
    }

    n = (sign * Math.floor( Math.abs(n) )) % Math.pow(2,32);
    if ( sign == -1 ) {
        n = ( n < -Math.pow(2,31) ) ? n + Math.pow(2,32) : n;
    } else{
        n = ( n >= Math.pow(2,31) ) ? n - Math.pow(2,32) : n;
    }

    return ( n );
}
function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "0 << 0",                        0,              0 << 0 );
    array[item++] = Assert.expectEq(    "-0 << 0",                       0,              -0 << 0 );
    array[item++] = Assert.expectEq(    "Infinity << 0",                 0,              "Infinity" << 0 );
    array[item++] = Assert.expectEq(    "-Infinity << 0",                0,              "-Infinity" << 0 );
    array[item++] = Assert.expectEq(    "Number.POSITIVE_INFINITY << 0", 0,              Number.POSITIVE_INFINITY << 0 );
    array[item++] = Assert.expectEq(    "Number.NEGATIVE_INFINITY << 0", 0,              Number.NEGATIVE_INFINITY << 0 );
    array[item++] = Assert.expectEq(    "Number.NaN << 0",               0,              Number.NaN << 0 );

    array[item++] = Assert.expectEq(    "Number.MIN_VALUE << 0",         0,              Number.MIN_VALUE << 0 );
    array[item++] = Assert.expectEq(    "-Number.MIN_VALUE << 0",        0,              -Number.MIN_VALUE << 0 );
    array[item++] = Assert.expectEq(    "0.1 << 0",                      0,              0.1 << 0 );
    array[item++] = Assert.expectEq(    "-0.1 << 0",                     0,              -0.1 << 0 );
    array[item++] = Assert.expectEq(    "1 << 0",                        1,              1 << 0 );
    array[item++] = Assert.expectEq(    "1.1 << 0",                      1,              1.1 << 0 );
    array[item++] = Assert.expectEq(    "-1 << 0",                     ToInt32(-1),             -1 << 0 );


    array[item++] = Assert.expectEq(    "2147483647 << 0",     ToInt32(2147483647),    2147483647 << 0 );
    array[item++] = Assert.expectEq(    "2147483648 << 0",     ToInt32(2147483648),    2147483648 << 0 );
    array[item++] = Assert.expectEq(    "2147483649 << 0",     ToInt32(2147483649),    2147483649 << 0 );

    array[item++] = Assert.expectEq(    "(Math.pow(2,31)-1) << 0", ToInt32(2147483647),    (Math.pow(2,31)-1) << 0 );
    array[item++] = Assert.expectEq(    "Math.pow(2,31) << 0",     ToInt32(2147483648),    Math.pow(2,31) << 0 );
    array[item++] = Assert.expectEq(    "(Math.pow(2,31)+1) << 0", ToInt32(2147483649),    (Math.pow(2,31)+1) << 0 );

    array[item++] = Assert.expectEq(    "(Math.pow(2,32)-1) << 0",   ToInt32(4294967295),    (Math.pow(2,32)-1) << 0 );
    array[item++] = Assert.expectEq(    "(Math.pow(2,32)) << 0",     ToInt32(4294967296),    (Math.pow(2,32)) << 0 );
    array[item++] = Assert.expectEq(    "(Math.pow(2,32)+1) << 0",   ToInt32(4294967297),    (Math.pow(2,32)+1) << 0 );

    array[item++] = Assert.expectEq(    "4294967295 << 0",     ToInt32(4294967295),    4294967295 << 0 );
    array[item++] = Assert.expectEq(    "4294967296 << 0",     ToInt32(4294967296),    4294967296 << 0 );
    array[item++] = Assert.expectEq(    "4294967297 << 0",     ToInt32(4294967297),    4294967297 << 0 );

    array[item++] = Assert.expectEq(    "'2147483647' << 0",   ToInt32(2147483647),    '2147483647' << 0 );
    array[item++] = Assert.expectEq(    "'2147483648' << 0",   ToInt32(2147483648),    '2147483648' << 0 );
    array[item++] = Assert.expectEq(    "'2147483649' << 0",   ToInt32(2147483649),    '2147483649' << 0 );

    array[item++] = Assert.expectEq(    "'4294967295' << 0",   ToInt32(4294967295),    '4294967295' << 0 );
    array[item++] = Assert.expectEq(    "'4294967296' << 0",   ToInt32(4294967296),    '4294967296' << 0 );
    array[item++] = Assert.expectEq(    "'4294967297' << 0",   ToInt32(4294967297),    '4294967297' << 0 );

    array[item++] = Assert.expectEq(    "-2147483647 << 0",    ToInt32(-2147483647),   -2147483647 << 0 );
    array[item++] = Assert.expectEq(    "-2147483648 << 0",    ToInt32(-2147483648),   -2147483648 << 0 );
    array[item++] = Assert.expectEq(    "-2147483649 << 0",    ToInt32(-2147483649),   -2147483649 << 0 );

    array[item++] = Assert.expectEq(    "-4294967295 << 0",    ToInt32(-4294967295),   -4294967295 << 0 );
    array[item++] = Assert.expectEq(    "-4294967296 << 0",    ToInt32(-4294967296),   -4294967296 << 0 );
    array[item++] = Assert.expectEq(    "-4294967297 << 0",    ToInt32(-4294967297),   -4294967297 << 0 );

    /*
     * Numbers between 2^31 and 2^32 will have a negative ToInt32 per ECMA (see step 5 of introduction)
     * (These are by stevechapel@earthlink.net; cf. http://bugzilla.mozilla.org/show_bug.cgi?id=120083)
     */
    array[item++] = Assert.expectEq(    "2147483648.25 << 0",  ToInt32(2147483648.25),   2147483648.25 << 0 );
    array[item++] = Assert.expectEq(    "2147483648.5 << 0",   ToInt32(2147483648.5),    2147483648.5 << 0 );
    array[item++] = Assert.expectEq(    "2147483648.75 << 0",  ToInt32(2147483648.75),   2147483648.75 << 0 );
    array[item++] = Assert.expectEq(    "4294967295.25 << 0",  ToInt32(4294967295.25),   4294967295.25 << 0 );
    array[item++] = Assert.expectEq(    "4294967295.5 << 0",   ToInt32(4294967295.5),    4294967295.5 << 0 );
    array[item++] = Assert.expectEq(    "4294967295.75 << 0",  ToInt32(4294967295.75),   4294967295.75 << 0 );
    array[item++] = Assert.expectEq(    "3000000000.25 << 0",  ToInt32(3000000000.25),   3000000000.25 << 0 );
    array[item++] = Assert.expectEq(    "3000000000.5 << 0",   ToInt32(3000000000.5),    3000000000.5 << 0 );
    array[item++] = Assert.expectEq(    "3000000000.75 << 0",  ToInt32(3000000000.75),   3000000000.75 << 0 );

    /*
     * Numbers between - 2^31 and - 2^32
     */
    array[item++] = Assert.expectEq(    "-2147483648.25 << 0",  ToInt32(-2147483648.25),   -2147483648.25 << 0 );
    array[item++] = Assert.expectEq(    "-2147483648.5 << 0",   ToInt32(-2147483648.5),    -2147483648.5 << 0 );
    array[item++] = Assert.expectEq(    "-2147483648.75 << 0",  ToInt32(-2147483648.75),   -2147483648.75 << 0 );
    array[item++] = Assert.expectEq(    "-4294967295.25 << 0",  ToInt32(-4294967295.25),   -4294967295.25 << 0 );
    array[item++] = Assert.expectEq(    "-4294967295.5 << 0",   ToInt32(-4294967295.5),    -4294967295.5 << 0 );
    array[item++] = Assert.expectEq(    "-4294967295.75 << 0",  ToInt32(-4294967295.75),   -4294967295.75 << 0 );
    array[item++] = Assert.expectEq(    "-3000000000.25 << 0",  ToInt32(-3000000000.25),   -3000000000.25 << 0 );
    array[item++] = Assert.expectEq(    "-3000000000.5 << 0",   ToInt32(-3000000000.5),    -3000000000.5 << 0 );
    array[item++] = Assert.expectEq(    "-3000000000.75 << 0",  ToInt32(-3000000000.75),   -3000000000.75 << 0 );

    return ( array );
}
