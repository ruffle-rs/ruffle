/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "e11_4_6";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();
    var BUGNUMBER="77391";


function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq(   "+('a string')",           Number.NaN,      +("a string") );
    array[item++] = Assert.expectEq(   "+('')",           0,      +("") );
    array[item++] = Assert.expectEq(   "+(' ')",          0,      +(" ") );
    array[item++] = Assert.expectEq(   "+(\\t)",          0,      +("\t") );
    array[item++] = Assert.expectEq(   "+(\\n)",          0,      +("\n") );
    array[item++] = Assert.expectEq(   "+(\\r)",          0,      +("\r") );
    array[item++] = Assert.expectEq(   "+(\\f)",          0,      +("\f") );

    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x0009)",   0,  +(String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x0020)",   0,  +(String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x000C)",   0,  +(String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x000B)",   0,  +(String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x000D)",   0,  +(String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "+(String.fromCharCode(0x000A)",   0,  +(String.fromCharCode(0x000A)) );

    //  a StringNumericLiteral may be preceeded or followed by whitespace and/or
    //  line terminators

    array[item++] = Assert.expectEq(   "+( '   ' +  999 )",        999,    +( '   '+999) );
    array[item++] = Assert.expectEq(   "+( '\\n'  + 999 )",       999,    +( '\n' +999) );
    array[item++] = Assert.expectEq(   "+( '\\r'  + 999 )",       999,    +( '\r' +999) );
    array[item++] = Assert.expectEq(   "+( '\\t'  + 999 )",       999,    +( '\t' +999) );
    array[item++] = Assert.expectEq(   "+( '\\f'  + 999 )",       999,    +( '\f' +999) );

    array[item++] = Assert.expectEq(   "+( 999 + '   ' )",        999,    +( 999+'   ') );
    array[item++] = Assert.expectEq(   "+( 999 + '\\n' )",        999,    +( 999+'\n' ) );
    array[item++] = Assert.expectEq(   "+( 999 + '\\r' )",        999,    +( 999+'\r' ) );
    array[item++] = Assert.expectEq(   "+( 999 + '\\t' )",        999,    +( 999+'\t' ) );
    array[item++] = Assert.expectEq(   "+( 999 + '\\f' )",        999,    +( 999+'\f' ) );

    array[item++] = Assert.expectEq(   "+( '\\n'  + 999 + '\\n' )",         999,    +( '\n' +999+'\n' ) );
    array[item++] = Assert.expectEq(   "+( '\\r'  + 999 + '\\r' )",         999,    +( '\r' +999+'\r' ) );
    array[item++] = Assert.expectEq(   "+( '\\t'  + 999 + '\\t' )",         999,    +( '\t' +999+'\t' ) );
    array[item++] = Assert.expectEq(   "+( '\\f'  + 999 + '\\f' )",         999,    +( '\f' +999+'\f' ) );

    array[item++] = Assert.expectEq(   "+( '   ' +  '999' )",     999,    +( '   '+'999') );
    array[item++] = Assert.expectEq(   "+( '\\n'  + '999' )",       999,    +( '\n' +'999') );
    array[item++] = Assert.expectEq(   "+( '\\r'  + '999' )",       999,    +( '\r' +'999') );
    array[item++] = Assert.expectEq(   "+( '\\t'  + '999' )",       999,    +( '\t' +'999') );
    array[item++] = Assert.expectEq(   "+( '\\f'  + '999' )",       999,    +( '\f' +'999') );

    array[item++] = Assert.expectEq(   "+( '999' + '   ' )",        999,    +( '999'+'   ') );
    array[item++] = Assert.expectEq(   "+( '999' + '\\n' )",        999,    +( '999'+'\n' ) );
    array[item++] = Assert.expectEq(   "+( '999' + '\\r' )",        999,    +( '999'+'\r' ) );
    array[item++] = Assert.expectEq(   "+( '999' + '\\t' )",        999,    +( '999'+'\t' ) );
    array[item++] = Assert.expectEq(   "+( '999' + '\\f' )",        999,    +( '999'+'\f' ) );

    array[item++] = Assert.expectEq(   "+( '\\n'  + '999' + '\\n' )",         999,    +( '\n' +'999'+'\n' ) );
    array[item++] = Assert.expectEq(   "+( '\\r'  + '999' + '\\r' )",         999,    +( '\r' +'999'+'\r' ) );
    array[item++] = Assert.expectEq(   "+( '\\t'  + '999' + '\\t' )",         999,    +( '\t' +'999'+'\t' ) );
    array[item++] = Assert.expectEq(   "+( '\\f'  + '999' + '\\f' )",         999,    +( '\f' +'999'+'\f' ) );

    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0009) +  '99' )",    99,     +( String.fromCharCode(0x0009) +  '99' ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0020) +  '99' )",    99,     +( String.fromCharCode(0x0020) +  '99' ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000C) +  '99' )",    99,     +( String.fromCharCode(0x000C) +  '99' ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000B) +  '99' )",    99,     +( String.fromCharCode(0x000B) +  '99' ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000D) +  '99' )",    99,     +( String.fromCharCode(0x000D) +  '99' ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000A) +  '99' )",    99,     +( String.fromCharCode(0x000A) +  '99' ) );

    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0009)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0020) +  '99' + String.fromCharCode(0x0020)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000C) +  '99' + String.fromCharCode(0x000C)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000D) +  '99' + String.fromCharCode(0x000D)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000B) +  '99' + String.fromCharCode(0x000B)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000A) +  '99' + String.fromCharCode(0x000A)",    99,     +( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x0009)",    99,     +( '99' + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x0020)",    99,     +( '99' + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x000C)",    99,     +( '99' + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x000D)",    99,     +( '99' + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x000B)",    99,     +( '99' + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "+( '99' + String.fromCharCode(0x000A)",    99,     +( '99' + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0009) +  99 )",    99,     +( String.fromCharCode(0x0009) +  99 ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0020) +  99 )",    99,     +( String.fromCharCode(0x0020) +  99 ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000C) +  99 )",    99,     +( String.fromCharCode(0x000C) +  99 ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000B) +  99 )",    99,     +( String.fromCharCode(0x000B) +  99 ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000D) +  99 )",    99,     +( String.fromCharCode(0x000D) +  99 ) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000A) +  99 )",    99,     +( String.fromCharCode(0x000A) +  99 ) );

    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0009)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x0020) +  99 + String.fromCharCode(0x0020)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000C) +  99 + String.fromCharCode(0x000C)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000D) +  99 + String.fromCharCode(0x000D)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000B) +  99 + String.fromCharCode(0x000B)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "+( String.fromCharCode(0x000A) +  99 + String.fromCharCode(0x000A)",    99,     +( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x0009)",    99,     +( 99 + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x0020)",    99,     +( 99 + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x000C)",    99,     +( 99 + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x000D)",    99,     +( 99 + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x000B)",    99,     +( 99 + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "+( 99 + String.fromCharCode(0x000A)",    99,     +( 99 + String.fromCharCode(0x000A)) );


    // StrNumericLiteral:::StrDecimalLiteral:::Infinity

    array[item++] = Assert.expectEq(   "+('Infinity')",   Math.pow(10,10000),   +("Infinity") );
    array[item++] = Assert.expectEq(   "+('-Infinity')", -Math.pow(10,10000),   +("-Infinity") );
    array[item++] = Assert.expectEq(   "+('+Infinity')",  Math.pow(10,10000),   +("+Infinity") );

    // StrNumericLiteral:::   StrDecimalLiteral ::: DecimalDigits . DecimalDigits opt ExponentPart opt

    array[item++] = Assert.expectEq(   "+('0')",          0,          +("0") );
    array[item++] = Assert.expectEq(   "+('-0')",         -0,         +("-0") );
    array[item++] = Assert.expectEq(   "+('+0')",          0,         +("+0") );

    array[item++] = Assert.expectEq(   "+('1')",          1,          +("1") );
    array[item++] = Assert.expectEq(   "+('-1')",         -1,         +("-1") );
    array[item++] = Assert.expectEq(   "+('+1')",          1,         +("+1") );

    array[item++] = Assert.expectEq(   "+('2')",          2,          +("2") );
    array[item++] = Assert.expectEq(   "+('-2')",         -2,         +("-2") );
    array[item++] = Assert.expectEq(   "+('+2')",          2,         +("+2") );

    array[item++] = Assert.expectEq(   "+('3')",          3,          +("3") );
    array[item++] = Assert.expectEq(   "+('-3')",         -3,         +("-3") );
    array[item++] = Assert.expectEq(   "+('+3')",          3,         +("+3") );

    array[item++] = Assert.expectEq(   "+('4')",          4,          +("4") );
    array[item++] = Assert.expectEq(   "+('-4')",         -4,         +("-4") );
    array[item++] = Assert.expectEq(   "+('+4')",          4,         +("+4") );

    array[item++] = Assert.expectEq(   "+('5')",          5,          +("5") );
    array[item++] = Assert.expectEq(   "+('-5')",         -5,         +("-5") );
    array[item++] = Assert.expectEq(   "+('+5')",          5,         +("+5") );

    array[item++] = Assert.expectEq(   "+('6')",          6,          +("6") );
    array[item++] = Assert.expectEq(   "+('-6')",         -6,         +("-6") );
    array[item++] = Assert.expectEq(   "+('+6')",          6,         +("+6") );

    array[item++] = Assert.expectEq(   "+('7')",          7,          +("7") );
    array[item++] = Assert.expectEq(   "+('-7')",         -7,         +("-7") );
    array[item++] = Assert.expectEq(   "+('+7')",          7,         +("+7") );

    array[item++] = Assert.expectEq(   "+('8')",          8,          +("8") );
    array[item++] = Assert.expectEq(   "+('-8')",         -8,         +("-8") );
    array[item++] = Assert.expectEq(   "+('+8')",          8,         +("+8") );

    array[item++] = Assert.expectEq(   "+('9')",          9,          +("9") );
    array[item++] = Assert.expectEq(   "+('-9')",         -9,         +("-9") );
    array[item++] = Assert.expectEq(   "+('+9')",          9,         +("+9") );

    array[item++] = Assert.expectEq(   "+('3.14159')",    3.14159,    +("3.14159") );
    array[item++] = Assert.expectEq(   "+('-3.14159')",   -3.14159,   +("-3.14159") );
    array[item++] = Assert.expectEq(   "+('+3.14159')",   3.14159,    +("+3.14159") );

    array[item++] = Assert.expectEq(   "+('3.')",         3,          +("3.") );
    array[item++] = Assert.expectEq(   "+('-3.')",        -3,         +("-3.") );
    array[item++] = Assert.expectEq(   "+('+3.')",        3,          +("+3.") );

    array[item++] = Assert.expectEq(   "+('3.e1')",       30,         +("3.e1") );
    array[item++] = Assert.expectEq(   "+('-3.e1')",      -30,        +("-3.e1") );
    array[item++] = Assert.expectEq(   "+('+3.e1')",      30,         +("+3.e1") );

    array[item++] = Assert.expectEq(   "+('3.e+1')",       30,         +("3.e+1") );
    array[item++] = Assert.expectEq(   "+('-3.e+1')",      -30,        +("-3.e+1") );
    array[item++] = Assert.expectEq(   "+('+3.e+1')",      30,         +("+3.e+1") );

    array[item++] = Assert.expectEq(   "+('3.e-1')",       .30,         +("3.e-1") );
    array[item++] = Assert.expectEq(   "+('-3.e-1')",      -.30,        +("-3.e-1") );
    array[item++] = Assert.expectEq(   "+('+3.e-1')",      .30,         +("+3.e-1") );

    // StrDecimalLiteral:::  .DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "+('.00001')",     0.00001,    +(".00001") );
    array[item++] = Assert.expectEq(   "+('+.00001')",    0.00001,    +("+.00001") );
    array[item++] = Assert.expectEq(   "+('-0.0001')",    -0.00001,   +("-.00001") );

    array[item++] = Assert.expectEq(   "+('.01e2')",      1,          +(".01e2") );
    array[item++] = Assert.expectEq(   "+('+.01e2')",     1,          +("+.01e2") );
    array[item++] = Assert.expectEq(   "+('-.01e2')",     -1,         +("-.01e2") );

    array[item++] = Assert.expectEq(   "+('.01e+2')",      1,         +(".01e+2") );
    array[item++] = Assert.expectEq(   "+('+.01e+2')",     1,         +("+.01e+2") );
    array[item++] = Assert.expectEq(   "+('-.01e+2')",     -1,        +("-.01e+2") );

    array[item++] = Assert.expectEq(   "+('.01e-2')",      0.0001,    +(".01e-2") );
    array[item++] = Assert.expectEq(   "+('+.01e-2')",     0.0001,    +("+.01e-2") );
    array[item++] = Assert.expectEq(   "+('-.01e-2')",     -0.0001,   +("-.01e-2") );

    //  StrDecimalLiteral:::    DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "+('1234e5')",     123400000,  +("1234e5") );
    array[item++] = Assert.expectEq(   "+('+1234e5')",    123400000,  +("+1234e5") );
    array[item++] = Assert.expectEq(   "+('-1234e5')",    -123400000, +("-1234e5") );

    array[item++] = Assert.expectEq(   "+('1234e+5')",    123400000,  +("1234e+5") );
    array[item++] = Assert.expectEq(   "+('+1234e+5')",   123400000,  +("+1234e+5") );
    array[item++] = Assert.expectEq(   "+('-1234e+5')",   -123400000, +("-1234e+5") );

    array[item++] = Assert.expectEq(   "+('1234e-5')",     0.01234,  +("1234e-5") );
    array[item++] = Assert.expectEq(   "+('+1234e-5')",    0.01234,  +("+1234e-5") );
    array[item++] = Assert.expectEq(   "+('-1234e-5')",    -0.01234, +("-1234e-5") );

    // StrNumericLiteral::: HexIntegerLiteral

    array[item++] = Assert.expectEq(   "+('0x0')",        0,          +("0x0"));
    array[item++] = Assert.expectEq(   "+('0x1')",        1,          +("0x1"));
    array[item++] = Assert.expectEq(   "+('0x2')",        2,          +("0x2"));
    array[item++] = Assert.expectEq(   "+('0x3')",        3,          +("0x3"));
    array[item++] = Assert.expectEq(   "+('0x4')",        4,          +("0x4"));
    array[item++] = Assert.expectEq(   "+('0x5')",        5,          +("0x5"));
    array[item++] = Assert.expectEq(   "+('0x6')",        6,          +("0x6"));
    array[item++] = Assert.expectEq(   "+('0x7')",        7,          +("0x7"));
    array[item++] = Assert.expectEq(   "+('0x8')",        8,          +("0x8"));
    array[item++] = Assert.expectEq(   "+('0x9')",        9,          +("0x9"));
    array[item++] = Assert.expectEq(   "+('0xa')",        10,         +("0xa"));
    array[item++] = Assert.expectEq(   "+('0xb')",        11,         +("0xb"));
    array[item++] = Assert.expectEq(   "+('0xc')",        12,         +("0xc"));
    array[item++] = Assert.expectEq(   "+('0xd')",        13,         +("0xd"));
    array[item++] = Assert.expectEq(   "+('0xe')",        14,         +("0xe"));
    array[item++] = Assert.expectEq(   "+('0xf')",        15,         +("0xf"));
    array[item++] = Assert.expectEq(   "+('0xA')",        10,         +("0xA"));
    array[item++] = Assert.expectEq(   "+('0xB')",        11,         +("0xB"));
    array[item++] = Assert.expectEq(   "+('0xC')",        12,         +("0xC"));
    array[item++] = Assert.expectEq(   "+('0xD')",        13,         +("0xD"));
    array[item++] = Assert.expectEq(   "+('0xE')",        14,         +("0xE"));
    array[item++] = Assert.expectEq(   "+('0xF')",        15,         +("0xF"));

    array[item++] = Assert.expectEq(   "+('0X0')",        0,          +("0X0"));
    array[item++] = Assert.expectEq(   "+('0X1')",        1,          +("0X1"));
    array[item++] = Assert.expectEq(   "+('0X2')",        2,          +("0X2"));
    array[item++] = Assert.expectEq(   "+('0X3')",        3,          +("0X3"));
    array[item++] = Assert.expectEq(   "+('0X4')",        4,          +("0X4"));
    array[item++] = Assert.expectEq(   "+('0X5')",        5,          +("0X5"));
    array[item++] = Assert.expectEq(   "+('0X6')",        6,          +("0X6"));
    array[item++] = Assert.expectEq(   "+('0X7')",        7,          +("0X7"));
    array[item++] = Assert.expectEq(   "+('0X8')",        8,          +("0X8"));
    array[item++] = Assert.expectEq(   "+('0X9')",        9,          +("0X9"));
    array[item++] = Assert.expectEq(   "+('0Xa')",        10,         +("0Xa"));
    array[item++] = Assert.expectEq(   "+('0Xb')",        11,         +("0Xb"));
    array[item++] = Assert.expectEq(   "+('0Xc')",        12,         +("0Xc"));
    array[item++] = Assert.expectEq(   "+('0Xd')",        13,         +("0Xd"));
    array[item++] = Assert.expectEq(   "+('0Xe')",        14,         +("0Xe"));
    array[item++] = Assert.expectEq(   "+('0Xf')",        15,         +("0Xf"));
    array[item++] = Assert.expectEq(   "+('0XA')",        10,         +("0XA"));
    array[item++] = Assert.expectEq(   "+('0XB')",        11,         +("0XB"));
    array[item++] = Assert.expectEq(   "+('0XC')",        12,         +("0XC"));
    array[item++] = Assert.expectEq(   "+('0XD')",        13,         +("0XD"));
    array[item++] = Assert.expectEq(   "+('0XE')",        14,         +("0XE"));
    array[item++] = Assert.expectEq(   "+('0XF')",        15,         +("0XF"));

    return array;

}
