/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11_13_2_4";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // If either operand is NaN,  result is NaN
    VAR1 = Number.NaN; VAR2=1;
    array[item++] = Assert.expectEq(     "VAR1 = NaN; VAR2=1; VAR1 += VAR2",       Number.NaN,  VAR1 += VAR2 );
    VAR1 = Number.NaN; VAR2=1; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = NaN; VAR2=1; VAR1 += VAR2; VAR1", Number.NaN,  VAR1 );
    VAR1 = Number.NaN; VAR2=0;
    array[item++] = Assert.expectEq(     "VAR1 = NaN; VAR2=0; VAR1 += VAR2",       Number.NaN, VAR1 += VAR2 );
    VAR1 = Number.NaN; VAR2=0; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = NaN; VAR2=0; VAR1 += VAR2; VAR1", Number.NaN,  VAR1 );
    VAR1 = 0; VAR2=Number.NaN;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2=NaN; VAR1 += VAR2",       Number.NaN, VAR1 += VAR2 );
    VAR1 = 0; VAR2=Number.NaN; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2=NaN; VAR1 += VAR2; VAR1", Number.NaN,  VAR1 );

    // the sum of two Infinities the same sign is the infinity of that sign
    // the sum of two Infinities of opposite sign is NaN
    VAR1 = Number.POSITIVE_INFINITY; VAR2 = Number.POSITIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = Infinity; VAR2= Infinity; VAR1 += VAR2; VAR1",   Number.POSITIVE_INFINITY,         VAR1 );
    VAR1 = Number.POSITIVE_INFINITY; VAR2 = Number.NEGATIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = Infinity; VAR2= -Infinity; VAR1 += VAR2; VAR1",  Number.NaN,                       VAR1 );
    VAR1 = Number.NEGATIVE_INFINITY; VAR2 = Number.POSITIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 =-Infinity; VAR2= Infinity; VAR1 += VAR2; VAR1",   Number.NaN,                       VAR1 );
    VAR1 = Number.NEGATIVE_INFINITY; VAR2 = Number.NEGATIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 =-Infinity; VAR2=-Infinity; VAR1 += VAR2; VAR1",   Number.NEGATIVE_INFINITY,         VAR1 );

    // the sum of an infinity and a finite value is equal to the infinite operand
    VAR1 = 0; VAR2 = Number.POSITIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= Infinity; VAR1 += VAR2;VAR1",    Number.POSITIVE_INFINITY,       VAR1 );
    VAR1 = -0; VAR2 = Number.POSITIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= Infinity; VAR1 += VAR2;VAR1",   Number.POSITIVE_INFINITY,       VAR1 );
     VAR1 = -0; VAR2 = Number.NEGATIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= -Infinity; VAR1 += VAR2;VAR1",  Number.NEGATIVE_INFINITY,      VAR1 );
    VAR1 = 0; VAR2 = Number.NEGATIVE_INFINITY; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= -Infinity; VAR1 += VAR2;VAR1",   Number.NEGATIVE_INFINITY,       VAR1 );

    // the sum of two negative zeros is -0. the sum of two positive zeros, or of two zeros of opposite sign, is +0
    VAR1 = 0; VAR2 = 0; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= 0; VAR1 += VAR2",    0,      VAR1 );
     VAR1 = 0; VAR2 = -0; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= -0; VAR1 += VAR2",   0,      VAR1 );
    VAR1 = -0; VAR2 = 0; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= 0; VAR1 += VAR2",   0,       VAR1 );
    VAR1 = -0; VAR2 = -0; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= -0; VAR1 += VAR2",  -0,       VAR1 );

    //  the sum of a zero and a nonzero finite value is eqal to the nonzero operand
    VAR1 = 0; VAR2 = 1; VAR2 += VAR1;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= 1; VAR2 += VAR1; VAR2",    1,       VAR2 );
    VAR1 = -0; VAR2 = 1; VAR2 += VAR1;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= 1; VAR2 += VAR1; VAR2",   1,       VAR2 );
    VAR1 = -0; VAR2 = -1; VAR2 += VAR1;
    array[item++] = Assert.expectEq(     "VAR1 = -0; VAR2= -1; VAR2 += VAR1; VAR2",  -1,      VAR2 );
    VAR1 = 0; VAR2 = -1; VAR2 += VAR1;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2= -1; VAR2 += VAR1; VAR2",   -1,       VAR2 );

    // the sum of a zero and a nozero finite value is equal to the nonzero operand.
     VAR1 = 0; VAR2=1;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2=1; VAR1 += VAR2",         1,         VAR1 += VAR2 );
      VAR1 = 0; VAR2=1; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = 0; VAR2=1; VAR1 += VAR2;VAR1",    1,        VAR1 );

    // the sum of two nonzero finite values of the same magnitude and opposite sign is +0
    VAR1 = Number.MAX_VALUE; VAR2= -Number.MAX_VALUE; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = Number.MAX_VALUE; VAR2= -Number.MAX_VALUE; VAR1 += VAR2; VAR1",    0,   VAR1 );
     VAR1 = Number.MIN_VALUE; VAR2= -Number.MIN_VALUE; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = Number.MIN_VALUE; VAR2= -Number.MIN_VALUE; VAR1 += VAR2; VAR1",   0, VAR1 );

/*
    VAR1 = 10; VAR2 = '0XFF';
    array[item++] = Assert.expectEq(     "VAR1 = 10; VAR2 = '0XFF', VAR1 += VAR2", 2550,       VAR1 += VAR2 );
    VAR1 = '0XFF'; VAR2 = 0XA;
    array[item++] = Assert.expectEq(     "VAR1 = '0xFF'; VAR2 = 0xA, VAR1 += VAR2", 2550,      VAR1 += VAR2 );
    VAR1 = '10'; VAR2 = '255';
    array[item++] = Assert.expectEq(     "VAR1 = '10'; VAR2 = '255', VAR1 += VAR2", 2550,      VAR1 += VAR2 );
    VAR1 = '10'; VAR2 = '0XFF';
    array[item++] = Assert.expectEq(     "VAR1 = '10'; VAR2 = '0XFF', VAR1 += VAR2", 2550,     VAR1 += VAR2 );
    VAR1 = '0XFF'; VAR2 = 0XA;
    array[item++] = Assert.expectEq(     "VAR1 = '0xFF'; VAR2 = 0xA, VAR1 += VAR2", 2550,      VAR1 += VAR2 );

    // boolean cases
    VAR1 = true; VAR2 = false;
    array[item++] = Assert.expectEq(     "VAR1 = true; VAR2 = false; VAR1 += VAR2",    0,      VAR1 += VAR2 );
     VAR1 = true; VAR2 = true;
    array[item++] = Assert.expectEq(     "VAR1 = true; VAR2 = true; VAR1 += VAR2",    1,      VAR1 += VAR2 );

    // object cases
    VAR1 = new Boolean(true); VAR2 = 10; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = new Boolean(true); VAR2 = 10; VAR1 += VAR2;VAR1",    10,       VAR1 );
    VAR1 = new Number(11); VAR2 = 10; VAR1 += VAR2;
    array[item++] = Assert.expectEq(     "VAR1 = new Number(11); VAR2 = 10; VAR1 += VAR2; VAR1",    110,       VAR1 );
    VAR1 = new Number(11); VAR2 = new Number(10);
    array[item++] = Assert.expectEq(     "VAR1 = new Number(11); VAR2 = new Number(10); VAR1 += VAR2",    110,       VAR1 += VAR2 );
    VAR1 = String('15'); VAR2 = new String('0xF');
    array[item++] = Assert.expectEq(     "VAR1 = new String('15'); VAR2 = new String('0xF'); VAR1 += VAR2",    255,       VAR1 += VAR2 );

*/
    return ( array );
}
