/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.3.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "ToNumber applied to the String type";
    var BUGNUMBER="77391";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    //  StringNumericLiteral:::StrWhiteSpace:::StrWhiteSpaceChar StrWhiteSpace:::
    //
    //  Name    Unicode Value   Escape Sequence
    //  <TAB>   0X0009          \t
    //  <SP>    0X0020
    //  <FF>    0X000C          \f
    //  <VT>    0X000B
    //  <CR>    0X000D          \r
    //  <LF>    0X000A          \n
    array[item++] = Assert.expectEq(   "Number('')",           0,      Number("") );
    array[item++] = Assert.expectEq(   "Number(' ')",          0,      Number(" ") );
    array[item++] = Assert.expectEq(   "Number(\\t)",          0,      Number("\t") );
    array[item++] = Assert.expectEq(   "Number(\\n)",          0,      Number("\n") );
    array[item++] = Assert.expectEq(   "Number(\\r)",          0,      Number("\r") );
    array[item++] = Assert.expectEq(   "Number(\\f)",          0,      Number("\f") );

    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x0009)",   0,  Number(String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x0020)",   0,  Number(String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x000C)",   0,  Number(String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x000B)",   0,  Number(String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x000D)",   0,  Number(String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "Number(String.fromCharCode(0x000A)",   0,  Number(String.fromCharCode(0x000A)) );

    //  a StringNumericLiteral may be preceeded or followed by whitespace and/or
    //  line terminators

    array[item++] = Assert.expectEq(   "Number( '   ' +  999 )",        999,    Number( '   '+999) );
    array[item++] = Assert.expectEq(   "Number( '\\n'  + 999 )",       999,    Number( '\n' +999) );
    array[item++] = Assert.expectEq(   "Number( '\\r'  + 999 )",       999,    Number( '\r' +999) );
    array[item++] = Assert.expectEq(   "Number( '\\t'  + 999 )",       999,    Number( '\t' +999) );
    array[item++] = Assert.expectEq(   "Number( '\\f'  + 999 )",       999,    Number( '\f' +999) );

    array[item++] = Assert.expectEq(   "Number( 999 + '   ' )",        999,    Number( 999+'   ') );
    array[item++] = Assert.expectEq(   "Number( 999 + '\\n' )",        999,    Number( 999+'\n' ) );
    array[item++] = Assert.expectEq(   "Number( 999 + '\\r' )",        999,    Number( 999+'\r' ) );
    array[item++] = Assert.expectEq(   "Number( 999 + '\\t' )",        999,    Number( 999+'\t' ) );
    array[item++] = Assert.expectEq(   "Number( 999 + '\\f' )",        999,    Number( 999+'\f' ) );

    array[item++] = Assert.expectEq(   "Number( '\\n'  + 999 + '\\n' )",         999,    Number( '\n' +999+'\n' ) );
    array[item++] = Assert.expectEq(   "Number( '\\r'  + 999 + '\\r' )",         999,    Number( '\r' +999+'\r' ) );
    array[item++] = Assert.expectEq(   "Number( '\\t'  + 999 + '\\t' )",         999,    Number( '\t' +999+'\t' ) );
    array[item++] = Assert.expectEq(   "Number( '\\f'  + 999 + '\\f' )",         999,    Number( '\f' +999+'\f' ) );

    array[item++] = Assert.expectEq(   "Number( '   ' +  '999' )",     999,    Number( '   '+'999') );
    array[item++] = Assert.expectEq(   "Number( '\\n'  + '999' )",       999,    Number( '\n' +'999') );
    array[item++] = Assert.expectEq(   "Number( '\\r'  + '999' )",       999,    Number( '\r' +'999') );
    array[item++] = Assert.expectEq(   "Number( '\\t'  + '999' )",       999,    Number( '\t' +'999') );
    array[item++] = Assert.expectEq(   "Number( '\\f'  + '999' )",       999,    Number( '\f' +'999') );

    array[item++] = Assert.expectEq(   "Number( '999' + '   ' )",        999,    Number( '999'+'   ') );
    array[item++] = Assert.expectEq(   "Number( '999' + '\\n' )",        999,    Number( '999'+'\n' ) );
    array[item++] = Assert.expectEq(   "Number( '999' + '\\r' )",        999,    Number( '999'+'\r' ) );
    array[item++] = Assert.expectEq(   "Number( '999' + '\\t' )",        999,    Number( '999'+'\t' ) );
    array[item++] = Assert.expectEq(   "Number( '999' + '\\f' )",        999,    Number( '999'+'\f' ) );

    array[item++] = Assert.expectEq(   "Number( '\\n'  + '999' + '\\n' )",         999,    Number( '\n' +'999'+'\n' ) );
    array[item++] = Assert.expectEq(   "Number( '\\r'  + '999' + '\\r' )",         999,    Number( '\r' +'999'+'\r' ) );
    array[item++] = Assert.expectEq(   "Number( '\\t'  + '999' + '\\t' )",         999,    Number( '\t' +'999'+'\t' ) );
    array[item++] = Assert.expectEq(   "Number( '\\f'  + '999' + '\\f' )",         999,    Number( '\f' +'999'+'\f' ) );

    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0009) +  '99' )",    99,     Number( String.fromCharCode(0x0009) +  '99' ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0020) +  '99' )",    99,     Number( String.fromCharCode(0x0020) +  '99' ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000C) +  '99' )",    99,     Number( String.fromCharCode(0x000C) +  '99' ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000B) +  '99' )",    99,     Number( String.fromCharCode(0x000B) +  '99' ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000D) +  '99' )",    99,     Number( String.fromCharCode(0x000D) +  '99' ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000A) +  '99' )",    99,     Number( String.fromCharCode(0x000A) +  '99' ) );

    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0009)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0020) +  '99' + String.fromCharCode(0x0020)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000C) +  '99' + String.fromCharCode(0x000C)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000D) +  '99' + String.fromCharCode(0x000D)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000B) +  '99' + String.fromCharCode(0x000B)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000A) +  '99' + String.fromCharCode(0x000A)",    99,     Number( String.fromCharCode(0x0009) +  '99' + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x0009)",    99,     Number( '99' + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x0020)",    99,     Number( '99' + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x000C)",    99,     Number( '99' + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x000D)",    99,     Number( '99' + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x000B)",    99,     Number( '99' + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "Number( '99' + String.fromCharCode(0x000A)",    99,     Number( '99' + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0009) +  99 )",    99,     Number( String.fromCharCode(0x0009) +  99 ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0020) +  99 )",    99,     Number( String.fromCharCode(0x0020) +  99 ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000C) +  99 )",    99,     Number( String.fromCharCode(0x000C) +  99 ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000B) +  99 )",    99,     Number( String.fromCharCode(0x000B) +  99 ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000D) +  99 )",    99,     Number( String.fromCharCode(0x000D) +  99 ) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000A) +  99 )",    99,     Number( String.fromCharCode(0x000A) +  99 ) );

    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0009)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x0020) +  99 + String.fromCharCode(0x0020)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000C) +  99 + String.fromCharCode(0x000C)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000D) +  99 + String.fromCharCode(0x000D)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000B) +  99 + String.fromCharCode(0x000B)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "Number( String.fromCharCode(0x000A) +  99 + String.fromCharCode(0x000A)",    99,     Number( String.fromCharCode(0x0009) +  99 + String.fromCharCode(0x000A)) );

    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x0009)",    99,     Number( 99 + String.fromCharCode(0x0009)) );
    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x0020)",    99,     Number( 99 + String.fromCharCode(0x0020)) );
    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x000C)",    99,     Number( 99 + String.fromCharCode(0x000C)) );
    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x000D)",    99,     Number( 99 + String.fromCharCode(0x000D)) );
    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x000B)",    99,     Number( 99 + String.fromCharCode(0x000B)) );
    array[item++] = Assert.expectEq(   "Number( 99 + String.fromCharCode(0x000A)",    99,     Number( 99 + String.fromCharCode(0x000A)) );


    // StrNumericLiteral:::StrDecimalLiteral:::Infinity

    array[item++] = Assert.expectEq(   "Number('Infinity')",   Math.pow(10,10000),   Number("Infinity") );
    array[item++] = Assert.expectEq(   "Number('-Infinity')", -Math.pow(10,10000),   Number("-Infinity") );
    array[item++] = Assert.expectEq(   "Number('+Infinity')",  Math.pow(10,10000),   Number("+Infinity") );

    // StrNumericLiteral:::   StrDecimalLiteral ::: DecimalDigits . DecimalDigits opt ExponentPart opt

    array[item++] = Assert.expectEq(   "Number('0')",          0,          Number("0") );
    array[item++] = Assert.expectEq(   "Number('-0')",         -0,         Number("-0") );
    array[item++] = Assert.expectEq(   "Number('+0')",          0,         Number("+0") );

    array[item++] = Assert.expectEq(   "Number('1')",          1,          Number("1") );
    array[item++] = Assert.expectEq(   "Number('-1')",         -1,         Number("-1") );
    array[item++] = Assert.expectEq(   "Number('+1')",          1,         Number("+1") );

    array[item++] = Assert.expectEq(   "Number('2')",          2,          Number("2") );
    array[item++] = Assert.expectEq(   "Number('-2')",         -2,         Number("-2") );
    array[item++] = Assert.expectEq(   "Number('+2')",          2,         Number("+2") );

    array[item++] = Assert.expectEq(   "Number('3')",          3,          Number("3") );
    array[item++] = Assert.expectEq(   "Number('-3')",         -3,         Number("-3") );
    array[item++] = Assert.expectEq(   "Number('+3')",          3,         Number("+3") );

    array[item++] = Assert.expectEq(   "Number('4')",          4,          Number("4") );
    array[item++] = Assert.expectEq(   "Number('-4')",         -4,         Number("-4") );
    array[item++] = Assert.expectEq(   "Number('+4')",          4,         Number("+4") );

    array[item++] = Assert.expectEq(   "Number('5')",          5,          Number("5") );
    array[item++] = Assert.expectEq(   "Number('-5')",         -5,         Number("-5") );
    array[item++] = Assert.expectEq(   "Number('+5')",          5,         Number("+5") );

    array[item++] = Assert.expectEq(   "Number('6')",          6,          Number("6") );
    array[item++] = Assert.expectEq(   "Number('-6')",         -6,         Number("-6") );
    array[item++] = Assert.expectEq(   "Number('+6')",          6,         Number("+6") );

    array[item++] = Assert.expectEq(   "Number('7')",          7,          Number("7") );
    array[item++] = Assert.expectEq(   "Number('-7')",         -7,         Number("-7") );
    array[item++] = Assert.expectEq(   "Number('+7')",          7,         Number("+7") );

    array[item++] = Assert.expectEq(   "Number('8')",          8,          Number("8") );
    array[item++] = Assert.expectEq(   "Number('-8')",         -8,         Number("-8") );
    array[item++] = Assert.expectEq(   "Number('+8')",          8,         Number("+8") );

    array[item++] = Assert.expectEq(   "Number('9')",          9,          Number("9") );
    array[item++] = Assert.expectEq(   "Number('-9')",         -9,         Number("-9") );
    array[item++] = Assert.expectEq(   "Number('+9')",          9,         Number("+9") );

    array[item++] = Assert.expectEq(   "Number('3.14159')",    3.14159,    Number("3.14159") );
    array[item++] = Assert.expectEq(   "Number('-3.14159')",   -3.14159,   Number("-3.14159") );
    array[item++] = Assert.expectEq(   "Number('+3.14159')",   3.14159,    Number("+3.14159") );

    array[item++] = Assert.expectEq(   "Number('3.')",         3,          Number("3.") );
    array[item++] = Assert.expectEq(   "Number('-3.')",        -3,         Number("-3.") );
    array[item++] = Assert.expectEq(   "Number('+3.')",        3,          Number("+3.") );

    array[item++] = Assert.expectEq(   "Number('3.e1')",       30,         Number("3.e1") );
    array[item++] = Assert.expectEq(   "Number('-3.e1')",      -30,        Number("-3.e1") );
    array[item++] = Assert.expectEq(   "Number('+3.e1')",      30,         Number("+3.e1") );

    array[item++] = Assert.expectEq(   "Number('3.e+1')",       30,         Number("3.e+1") );
    array[item++] = Assert.expectEq(   "Number('-3.e+1')",      -30,        Number("-3.e+1") );
    array[item++] = Assert.expectEq(   "Number('+3.e+1')",      30,         Number("+3.e+1") );

    array[item++] = Assert.expectEq(   "Number('3.e-1')",       .30,         Number("3.e-1") );
    array[item++] = Assert.expectEq(   "Number('-3.e-1')",      -.30,        Number("-3.e-1") );
    array[item++] = Assert.expectEq(   "Number('+3.e-1')",      .30,         Number("+3.e-1") );

    // StrDecimalLiteral:::  .DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "Number('.00001')",     0.00001,    Number(".00001") );
    array[item++] = Assert.expectEq(   "Number('+.00001')",    0.00001,    Number("+.00001") );
    array[item++] = Assert.expectEq(   "Number('-0.0001')",    -0.00001,   Number("-.00001") );

    array[item++] = Assert.expectEq(   "Number('.01e2')",      1,          Number(".01e2") );
    array[item++] = Assert.expectEq(   "Number('+.01e2')",     1,          Number("+.01e2") );
    array[item++] = Assert.expectEq(   "Number('-.01e2')",     -1,         Number("-.01e2") );

    array[item++] = Assert.expectEq(   "Number('.01e+2')",      1,         Number(".01e+2") );
    array[item++] = Assert.expectEq(   "Number('+.01e+2')",     1,         Number("+.01e+2") );
    array[item++] = Assert.expectEq(   "Number('-.01e+2')",     -1,        Number("-.01e+2") );

    array[item++] = Assert.expectEq(   "Number('.01e-2')",      0.0001,    Number(".01e-2") );
    array[item++] = Assert.expectEq(   "Number('+.01e-2')",     0.0001,    Number("+.01e-2") );
    array[item++] = Assert.expectEq(   "Number('-.01e-2')",     -0.0001,   Number("-.01e-2") );

    //  StrDecimalLiteral:::    DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "Number('1234e5')",     123400000,  Number("1234e5") );
    array[item++] = Assert.expectEq(   "Number('+1234e5')",    123400000,  Number("+1234e5") );
    array[item++] = Assert.expectEq(   "Number('-1234e5')",    -123400000, Number("-1234e5") );

    array[item++] = Assert.expectEq(   "Number('1234e+5')",    123400000,  Number("1234e+5") );
    array[item++] = Assert.expectEq(   "Number('+1234e+5')",   123400000,  Number("+1234e+5") );
    array[item++] = Assert.expectEq(   "Number('-1234e+5')",   -123400000, Number("-1234e+5") );

    array[item++] = Assert.expectEq(   "Number('1234e-5')",     0.01234,  Number("1234e-5") );
    array[item++] = Assert.expectEq(   "Number('+1234e-5')",    0.01234,  Number("+1234e-5") );
    array[item++] = Assert.expectEq(   "Number('-1234e-5')",    -0.01234, Number("-1234e-5") );

    // StrNumericLiteral::: HexIntegerLiteral

    array[item++] = Assert.expectEq(   "Number('0x0')",        0,          Number("0x0"));
    array[item++] = Assert.expectEq(   "Number('0x1')",        1,          Number("0x1"));
    array[item++] = Assert.expectEq(   "Number('0x2')",        2,          Number("0x2"));
    array[item++] = Assert.expectEq(   "Number('0x3')",        3,          Number("0x3"));
    array[item++] = Assert.expectEq(   "Number('0x4')",        4,          Number("0x4"));
    array[item++] = Assert.expectEq(   "Number('0x5')",        5,          Number("0x5"));
    array[item++] = Assert.expectEq(   "Number('0x6')",        6,          Number("0x6"));
    array[item++] = Assert.expectEq(   "Number('0x7')",        7,          Number("0x7"));
    array[item++] = Assert.expectEq(   "Number('0x8')",        8,          Number("0x8"));
    array[item++] = Assert.expectEq(   "Number('0x9')",        9,          Number("0x9"));
    array[item++] = Assert.expectEq(   "Number('0xa')",        10,         Number("0xa"));
    array[item++] = Assert.expectEq(   "Number('0xb')",        11,         Number("0xb"));
    array[item++] = Assert.expectEq(   "Number('0xc')",        12,         Number("0xc"));
    array[item++] = Assert.expectEq(   "Number('0xd')",        13,         Number("0xd"));
    array[item++] = Assert.expectEq(   "Number('0xe')",        14,         Number("0xe"));
    array[item++] = Assert.expectEq(   "Number('0xf')",        15,         Number("0xf"));
    array[item++] = Assert.expectEq(   "Number('0xA')",        10,         Number("0xA"));
    array[item++] = Assert.expectEq(   "Number('0xB')",        11,         Number("0xB"));
    array[item++] = Assert.expectEq(   "Number('0xC')",        12,         Number("0xC"));
    array[item++] = Assert.expectEq(   "Number('0xD')",        13,         Number("0xD"));
    array[item++] = Assert.expectEq(   "Number('0xE')",        14,         Number("0xE"));
    array[item++] = Assert.expectEq(   "Number('0xF')",        15,         Number("0xF"));

    array[item++] = Assert.expectEq(   "Number('0X0')",        0,          Number("0X0"));
    array[item++] = Assert.expectEq(   "Number('0X1')",        1,          Number("0X1"));
    array[item++] = Assert.expectEq(   "Number('0X2')",        2,          Number("0X2"));
    array[item++] = Assert.expectEq(   "Number('0X3')",        3,          Number("0X3"));
    array[item++] = Assert.expectEq(   "Number('0X4')",        4,          Number("0X4"));
    array[item++] = Assert.expectEq(   "Number('0X5')",        5,          Number("0X5"));
    array[item++] = Assert.expectEq(   "Number('0X6')",        6,          Number("0X6"));
    array[item++] = Assert.expectEq(   "Number('0X7')",        7,          Number("0X7"));
    array[item++] = Assert.expectEq(   "Number('0X8')",        8,          Number("0X8"));
    array[item++] = Assert.expectEq(   "Number('0X9')",        9,          Number("0X9"));
    array[item++] = Assert.expectEq(   "Number('0Xa')",        10,         Number("0Xa"));
    array[item++] = Assert.expectEq(   "Number('0Xb')",        11,         Number("0Xb"));
    array[item++] = Assert.expectEq(   "Number('0Xc')",        12,         Number("0Xc"));
    array[item++] = Assert.expectEq(   "Number('0Xd')",        13,         Number("0Xd"));
    array[item++] = Assert.expectEq(   "Number('0Xe')",        14,         Number("0Xe"));
    array[item++] = Assert.expectEq(   "Number('0Xf')",        15,         Number("0Xf"));
    array[item++] = Assert.expectEq(   "Number('0XA')",        10,         Number("0XA"));
    array[item++] = Assert.expectEq(   "Number('0XB')",        11,         Number("0XB"));
    array[item++] = Assert.expectEq(   "Number('0XC')",        12,         Number("0XC"));
    array[item++] = Assert.expectEq(   "Number('0XD')",        13,         Number("0XD"));
    array[item++] = Assert.expectEq(   "Number('0XE')",        14,         Number("0XE"));
    array[item++] = Assert.expectEq(   "Number('0XF')",        15,         Number("0XF"));

    return ( array );
}

