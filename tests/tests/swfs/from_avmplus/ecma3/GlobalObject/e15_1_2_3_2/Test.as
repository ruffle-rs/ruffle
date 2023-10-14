/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.1.2.3-2";
//     var VERSION = "ECMA_1";

    var BUGNUMBER = "77391";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "parseFloat(true)",      Number.NaN,     parseFloat(true) );
    array[item++] = Assert.expectEq(  "parseFloat(false)",     Number.NaN,     parseFloat(false) );
    array[item++] = Assert.expectEq(  "parseFloat('string')",  Number.NaN,     parseFloat("string") );

    array[item++] = Assert.expectEq(  "parseFloat('     Infinity')",      Number.POSITIVE_INFINITY,    parseFloat("Infinity") );
//    array[item++] = Assert.expectEq(  "parseFloat(Infinity)",      Number.POSITIVE_INFINITY,    parseFloat(Infinity) );

    array[item++] = Assert.expectEq(   "parseFloat('          0')",          0,          parseFloat("          0") );
    array[item++] = Assert.expectEq(   "parseFloat('          -0')",         -0,         parseFloat("          -0") );
    array[item++] = Assert.expectEq(   "parseFloat('          +0')",          0,         parseFloat("          +0") );

    array[item++] = Assert.expectEq(   "parseFloat('          1')",          1,          parseFloat("          1") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1')",         -1,         parseFloat("          -1") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1')",          1,         parseFloat("          +1") );

    array[item++] = Assert.expectEq(   "parseFloat('          2')",          2,          parseFloat("          2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -2')",         -2,         parseFloat("          -2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +2')",          2,         parseFloat("          +2") );

    array[item++] = Assert.expectEq(   "parseFloat('          3')",          3,          parseFloat("          3") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3')",         -3,         parseFloat("          -3") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3')",          3,         parseFloat("          +3") );

    array[item++] = Assert.expectEq(   "parseFloat('          4')",          4,          parseFloat("          4") );
    array[item++] = Assert.expectEq(   "parseFloat('          -4')",         -4,         parseFloat("          -4") );
    array[item++] = Assert.expectEq(   "parseFloat('          +4')",          4,         parseFloat("          +4") );

    array[item++] = Assert.expectEq(   "parseFloat('          5')",          5,          parseFloat("          5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -5')",         -5,         parseFloat("          -5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +5')",          5,         parseFloat("          +5") );

    array[item++] = Assert.expectEq(   "parseFloat('          6')",          6,          parseFloat("          6") );
    array[item++] = Assert.expectEq(   "parseFloat('          -6')",         -6,         parseFloat("          -6") );
    array[item++] = Assert.expectEq(   "parseFloat('          +6')",          6,         parseFloat("          +6") );

    array[item++] = Assert.expectEq(   "parseFloat('          7')",          7,          parseFloat("          7") );
    array[item++] = Assert.expectEq(   "parseFloat('          -7')",         -7,         parseFloat("          -7") );
    array[item++] = Assert.expectEq(   "parseFloat('          +7')",          7,         parseFloat("          +7") );

    array[item++] = Assert.expectEq(   "parseFloat('          8')",          8,          parseFloat("          8") );
    array[item++] = Assert.expectEq(   "parseFloat('          -8')",         -8,         parseFloat("          -8") );
    array[item++] = Assert.expectEq(   "parseFloat('          +8')",          8,         parseFloat("          +8") );

    array[item++] = Assert.expectEq(   "parseFloat('          9')",          9,          parseFloat("          9") );
    array[item++] = Assert.expectEq(   "parseFloat('          -9')",         -9,         parseFloat("          -9") );
    array[item++] = Assert.expectEq(   "parseFloat('          +9')",          9,         parseFloat("          +9") );

    array[item++] = Assert.expectEq(   "parseFloat('          3.14159')",    3.14159,    parseFloat("          3.14159") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3.14159')",   -3.14159,   parseFloat("          -3.14159") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3.14159')",   3.14159,    parseFloat("          +3.14159") );

    array[item++] = Assert.expectEq(   "parseFloat('          3.')",         3,          parseFloat("          3.") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3.')",        -3,         parseFloat("          -3.") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3.')",        3,          parseFloat("          +3.") );

    array[item++] = Assert.expectEq(   "parseFloat('          3.e1')",       30,         parseFloat("          3.e1") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3.e1')",      -30,        parseFloat("          -3.e1") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3.e1')",      30,         parseFloat("          +3.e1") );

    array[item++] = Assert.expectEq(   "parseFloat('          3.e+1')",       30,         parseFloat("          3.e+1") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3.e+1')",      -30,        parseFloat("          -3.e+1") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3.e+1')",      30,         parseFloat("          +3.e+1") );

    array[item++] = Assert.expectEq(   "parseFloat('          3.e-1')",       .30,         parseFloat("          3.e-1") );
    array[item++] = Assert.expectEq(   "parseFloat('          -3.e-1')",      -.30,        parseFloat("          -3.e-1") );
    array[item++] = Assert.expectEq(   "parseFloat('          +3.e-1')",      .30,         parseFloat("          +3.e-1") );

    // StrDecimalLiteral:::  .DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "parseFloat('          .00001')",     0.00001,    parseFloat("          .00001") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.00001')",    0.00001,    parseFloat("          +.00001") );
    array[item++] = Assert.expectEq(   "parseFloat('          -0.0001')",    -0.00001,   parseFloat("          -.00001") );

    array[item++] = Assert.expectEq(   "parseFloat('          .01e2')",      1,          parseFloat("          .01e2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01e2')",     1,          parseFloat("          +.01e2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01e2')",     -1,         parseFloat("          -.01e2") );

    array[item++] = Assert.expectEq(   "parseFloat('          .01e+2')",      1,         parseFloat("          .01e+2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01e+2')",     1,         parseFloat("          +.01e+2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01e+2')",     -1,        parseFloat("          -.01e+2") );

    array[item++] = Assert.expectEq(   "parseFloat('          .01e-2')",      0.0001,    parseFloat("          .01e-2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01e-2')",     0.0001,    parseFloat("          +.01e-2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01e-2')",     -0.0001,   parseFloat("          -.01e-2") );

    //  StrDecimalLiteral:::    DecimalDigits ExponentPart opt

    array[item++] = Assert.expectEq(   "parseFloat('          1234e5')",     123400000,  parseFloat("          1234e5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234e5')",    123400000,  parseFloat("          +1234e5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234e5')",    -123400000, parseFloat("          -1234e5") );

    array[item++] = Assert.expectEq(   "parseFloat('          1234e+5')",    123400000,  parseFloat("          1234e+5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234e+5')",   123400000,  parseFloat("          +1234e+5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234e+5')",   -123400000, parseFloat("          -1234e+5") );

    array[item++] = Assert.expectEq(   "parseFloat('          1234e-5')",     0.01234,  parseFloat("          1234e-5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234e-5')",    0.01234,  parseFloat("          +1234e-5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234e-5')",    -0.01234, parseFloat("          -1234e-5") );


    array[item++] = Assert.expectEq(   "parseFloat('          .01E2')",      1,          parseFloat("          .01E2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01E2')",     1,          parseFloat("          +.01E2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01E2')",     -1,         parseFloat("          -.01E2") );

    array[item++] = Assert.expectEq(   "parseFloat('          .01E+2')",      1,         parseFloat("          .01E+2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01E+2')",     1,         parseFloat("          +.01E+2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01E+2')",     -1,        parseFloat("          -.01E+2") );

    array[item++] = Assert.expectEq(   "parseFloat('          .01E-2')",      0.0001,    parseFloat("          .01E-2") );
    array[item++] = Assert.expectEq(   "parseFloat('          +.01E-2')",     0.0001,    parseFloat("          +.01E-2") );
    array[item++] = Assert.expectEq(   "parseFloat('          -.01E-2')",     -0.0001,   parseFloat("          -.01E-2") );

    //  StrDecimalLiteral:::    DecimalDigits ExponentPart opt
    array[item++] = Assert.expectEq(   "parseFloat('          1234E5')",     123400000,  parseFloat("          1234E5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234E5')",    123400000,  parseFloat("          +1234E5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234E5')",    -123400000, parseFloat("          -1234E5") );

    array[item++] = Assert.expectEq(   "parseFloat('          1234E+5')",    123400000,  parseFloat("          1234E+5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234E+5')",   123400000,  parseFloat("          +1234E+5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234E+5')",   -123400000, parseFloat("          -1234E+5") );

    array[item++] = Assert.expectEq(   "parseFloat('          1234E-5')",     0.01234,  parseFloat("          1234E-5") );
    array[item++] = Assert.expectEq(   "parseFloat('          +1234E-5')",    0.01234,  parseFloat("          +1234E-5") );
    array[item++] = Assert.expectEq(   "parseFloat('          -1234E-5')",    -0.01234, parseFloat("          -1234E-5") );


    // hex cases should all return NaN

    array[item++] = Assert.expectEq(   "parseFloat('          0x0')",        0,         parseFloat("          0x0"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x1')",        0,         parseFloat("          0x1"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x2')",        0,         parseFloat("          0x2"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x3')",        0,         parseFloat("          0x3"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x4')",        0,         parseFloat("          0x4"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x5')",        0,         parseFloat("          0x5"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x6')",        0,         parseFloat("          0x6"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x7')",        0,         parseFloat("          0x7"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x8')",        0,         parseFloat("          0x8"));
    array[item++] = Assert.expectEq(   "parseFloat('          0x9')",        0,         parseFloat("          0x9"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xa')",        0,         parseFloat("          0xa"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xb')",        0,         parseFloat("          0xb"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xc')",        0,         parseFloat("          0xc"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xd')",        0,         parseFloat("          0xd"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xe')",        0,         parseFloat("          0xe"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xf')",        0,         parseFloat("          0xf"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xA')",        0,         parseFloat("          0xA"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xB')",        0,         parseFloat("          0xB"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xC')",        0,         parseFloat("          0xC"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xD')",        0,         parseFloat("          0xD"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xE')",        0,         parseFloat("          0xE"));
    array[item++] = Assert.expectEq(   "parseFloat('          0xF')",        0,         parseFloat("          0xF"));

    array[item++] = Assert.expectEq(   "parseFloat('          0X0')",        0,         parseFloat("          0X0"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X1')",        0,         parseFloat("          0X1"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X2')",        0,         parseFloat("          0X2"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X3')",        0,         parseFloat("          0X3"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X4')",        0,         parseFloat("          0X4"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X5')",        0,         parseFloat("          0X5"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X6')",        0,         parseFloat("          0X6"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X7')",        0,         parseFloat("          0X7"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X8')",        0,         parseFloat("          0X8"));
    array[item++] = Assert.expectEq(   "parseFloat('          0X9')",        0,         parseFloat("          0X9"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xa')",        0,         parseFloat("          0Xa"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xb')",        0,         parseFloat("          0Xb"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xc')",        0,         parseFloat("          0Xc"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xd')",        0,         parseFloat("          0Xd"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xe')",        0,         parseFloat("          0Xe"));
    array[item++] = Assert.expectEq(   "parseFloat('          0Xf')",        0,         parseFloat("          0Xf"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XA')",        0,         parseFloat("          0XA"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XB')",        0,         parseFloat("          0XB"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XC')",        0,         parseFloat("          0XC"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XD')",        0,         parseFloat("          0XD"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XE')",        0,         parseFloat("          0XE"));
    array[item++] = Assert.expectEq(   "parseFloat('          0XF')",        0,         parseFloat("          0XF"));

    // A StringNumericLiteral may not use octal notation

    array[item++] = Assert.expectEq(   "parseFloat('          00')",        0,         parseFloat("          00"));
    array[item++] = Assert.expectEq(   "parseFloat('          01')",        1,         parseFloat("          01"));
    array[item++] = Assert.expectEq(   "parseFloat('          02')",        2,         parseFloat("          02"));
    array[item++] = Assert.expectEq(   "parseFloat('          03')",        3,         parseFloat("          03"));
    array[item++] = Assert.expectEq(   "parseFloat('          04')",        4,         parseFloat("          04"));
    array[item++] = Assert.expectEq(   "parseFloat('          05')",        5,         parseFloat("          05"));
    array[item++] = Assert.expectEq(   "parseFloat('          06')",        6,         parseFloat("          06"));
    array[item++] = Assert.expectEq(   "parseFloat('          07')",        7,         parseFloat("          07"));
    array[item++] = Assert.expectEq(   "parseFloat('          010')",       10,        parseFloat("          010"));
    array[item++] = Assert.expectEq(   "parseFloat('          011')",       11,        parseFloat("          011"));

    // A StringNumericLIteral may have any number of leading 0 digits

    array[item++] = Assert.expectEq(   "parseFloat('          001')",        1,         parseFloat("          001"));
    array[item++] = Assert.expectEq(   "parseFloat('          0001')",       1,         parseFloat("          0001"));

    // A StringNumericLIteral may have any number of leading 0 digits

    array[item++] = Assert.expectEq(   "parseFloat(001)",        1,         parseFloat(001));
    array[item++] = Assert.expectEq(   "parseFloat(0001)",       1,         parseFloat(0001));

    // make sure it'          s reflexive
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.PI+'          ')",      Math.PI,        parseFloat( '                    '          +Math.PI+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.LN2+'          ')",     Math.LN2,       parseFloat( '                    '          +Math.LN2+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.LN10+'          ')",    Math.LN10,      parseFloat( '                    '          +Math.LN10+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.LOG2E+'          ')",   Math.LOG2E,     parseFloat( '                    '          +Math.LOG2E+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.LOG10E+'          ')",  Math.LOG10E,    parseFloat( '                    '          +Math.LOG10E+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.SQRT2+'          ')",   Math.SQRT2,     parseFloat( '                    '          +Math.SQRT2+'          '));
    array[item++] = Assert.expectEq(   "parseFloat( '                    '          +Math.SQRT1_2+'          ')", Math.SQRT1_2,   parseFloat( '                    '          +Math.SQRT1_2+'          '));


    return ( array );
}
