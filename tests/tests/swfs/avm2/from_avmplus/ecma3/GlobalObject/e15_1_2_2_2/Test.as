/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "15.1.2.2-2";
// var VERSION = "ECMA_1";
// var TITLE   = "parseInt(string, radix)";
var BUGNUMBER="123874";

var tc = 0;


var testcases = new Array();

testcases[tc++] = Assert.expectEq( 
    'parseInt("000000100000000100100011010001010110011110001001101010111100",2)',
    9027215253084860,
    parseInt("000000100000000100100011010001010110011110001001101010111100",2) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("000000100000000100100011010001010110011110001001101010111101",2)',
    9027215253084860,
    parseInt("000000100000000100100011010001010110011110001001101010111101",2));

testcases[tc++] = Assert.expectEq( 
    'parseInt("000000100000000100100011010001010110011110001001101010111111",2)',
    9027215253084864,
    parseInt("000000100000000100100011010001010110011110001001101010111111",2) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0000001000000001001000110100010101100111100010011010101111010",2)',
    18054430506169720,
    parseInt("0000001000000001001000110100010101100111100010011010101111010",2) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0000001000000001001000110100010101100111100010011010101111011",2)',
    18054430506169724,
    parseInt("0000001000000001001000110100010101100111100010011010101111011",2));

testcases[tc++] = Assert.expectEq( 
    'parseInt("0000001000000001001000110100010101100111100010011010101111100",2)',
    18054430506169724,
    parseInt("0000001000000001001000110100010101100111100010011010101111100",2) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0000001000000001001000110100010101100111100010011010101111110",2)',
    18054430506169728,
    parseInt("0000001000000001001000110100010101100111100010011010101111110",2) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("yz",35)',
    34,
    parseInt("yz",35) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("yz",36)',
    1259,
    parseInt("yz",36) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("yz",37)',
    NaN,
    parseInt("yz",37) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("+77")',
    77,
    parseInt("+77") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("-77",9)',
    -70,
    parseInt("-77",9) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("\u20001234\u2000")',
    1234,
    parseInt("\u20001234\u2000") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("123456789012345678")',
    123456789012345700,
    parseInt("123456789012345678") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("9",8)',
    NaN,
    parseInt("9",8) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("1e2")',
    1,
    parseInt("1e2") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("1.9999999999999999999")',
    1,
    parseInt("1.9999999999999999999") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0x10")',
    16,
    parseInt("0x10") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0x10",10)',
    0,
    parseInt("0x10",10));

testcases[tc++] = Assert.expectEq( 
    'parseInt("0022")',
    22,
    parseInt("0022"));

testcases[tc++] = Assert.expectEq( 
    'parseInt("0022",10)',
    22,
    parseInt("0022",10) );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0x1000000000000080")',
    1152921504606847000,
    parseInt("0x1000000000000080") );

testcases[tc++] = Assert.expectEq( 
    'parseInt("0x1000000000000081")',
    1152921504606847200,
    parseInt("0x1000000000000081") );

s =
"0xFFFFFFFFFFFFF80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

s += "0000000000000000000000000000000000000";

testcases[tc++] = Assert.expectEq( 
    "s = " + s +"; -s",
    -1.7976931348623157e+308,
    -s );

s =
"0xFFFFFFFFFFFFF80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
s += "0000000000000000000000000000000000001";

testcases[tc++] = Assert.expectEq( 
    "s = " + s +"; -s",
    -1.7976931348623157e+308,
    -s );


s = "0xFFFFFFFFFFFFFC0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

s += "0000000000000000000000000000000000000"


testcases[tc++] = Assert.expectEq( 
    "s = " + s + "; -s",
    -Infinity,
    -s );

s = "0xFFFFFFFFFFFFFB0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
s += "0000000000000000000000000000000000001";

testcases[tc++] = Assert.expectEq( 
    "s = " + s + "; -s",
    -1.7976931348623157e+308,
    -s );

s += "0"

testcases[tc++] = Assert.expectEq( 
    "s = " + s + "; -s",
    -Infinity,
    -s );

testcases[tc++] = Assert.expectEq( 
    'parseInt(s)',
    Infinity,
    parseInt(s) );

testcases[tc++] = Assert.expectEq( 
    'parseInt(s,32)',
    0,
    parseInt(s,32) );

testcases[tc++] = Assert.expectEq( 
    'parseInt(s,36)',
    Infinity,
    parseInt(s,36));

