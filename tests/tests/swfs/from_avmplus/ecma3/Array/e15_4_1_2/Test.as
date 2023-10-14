/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4.1.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Constructor Called as a Function:  Array(len)";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "(Array()).length",             0,                              (Array()).length );
    array[item++] = Assert.expectEq(   "(Array(0)).length",            0,                              (Array(0)).length );
    array[item++] = Assert.expectEq(   "(Array(1)).length",            1,                              (Array(1)).length );
    array[item++] = Assert.expectEq(   "(Array(10)).length",           10,                             (Array(10)).length );
    array[item++] = Assert.expectEq(   "(Array('1')).length",          1,                              (Array('1')).length );
    array[item++] = Assert.expectEq(   "(Array(1000)).length",         1000,                           (Array(1000)).length );
    array[item++] = Assert.expectEq(   "(Array('1000')).length",       1,                              (Array('1000')).length );
    array[item++] = Assert.expectEq(   "(Array(4294967295)).length",   ToUint32(4294967295),           (Array(4294967295)).length );
    array[item++] = Assert.expectEq(   "(Array(Math.pow(2,31)-1)).length",     ToUint32(Math.pow(2,31)-1),     (Array(Math.pow(2,31)-1)).length );
    array[item++] = Assert.expectEq(   "(Array(Math.pow(2,31))).length",       ToUint32(Math.pow(2,31)),       (Array(Math.pow(2,31))).length );
    array[item++] = Assert.expectEq(   "(Array(Math.pow(2,31)+1)).length",     ToUint32(Math.pow(2,31)+1),     (Array(Math.pow(2,31)+1)).length );
    array[item++] = Assert.expectEq(   "(Array('8589934592')).length", 1,                              (Array("8589934592")).length );
    array[item++] = Assert.expectEq(   "(Array('4294967296')).length", 1,                              (Array("4294967296")).length );
    array[item++] = Assert.expectEq(   "(Array(1073741823)).length",   ToUint32(1073741823),           (Array(1073741823)).length );
    array[item++] = Assert.expectEq(   "(Array(1073741824)).length",   ToUint32(1073741824),           (Array(1073741824)).length );
    array[item++] = Assert.expectEq(   "(Array('a string')).length",   1,                              (Array("a string")).length );

    return ( array );
}

function ToUint32( n ) {
    n = Number( n );
    var sign = ( n < 0 ) ? -1 : 1;

    if ( Math.abs( n ) == 0 || Math.abs( n ) == Number.POSITIVE_INFINITY) {
        return 0;
    }
    n = sign * Math.floor( Math.abs(n) )

    n = n % Math.pow(2,32);

    if ( n < 0 ){
        n += Math.pow(2,32);
    }

    return ( n );
}
