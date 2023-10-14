/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.4.5.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array [[Put]] (P, V)";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();

    var item = 0;

    // P is "length"

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(); A.length = 1000; A.length",
                                    1000,
                                    (A = new Array(), A.length = 1000, A.length ) );

    // A has Property P, and P is not length or an array index
    array[item++] = Assert.expectEq(   
                                    "var A = new Array(1000); A.name = 'name of this array'; A.name",
                                    'name of this array',
                                    (A = new Array(1000), A.name = 'name of this array', A.name ) );

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(1000); A.name = 'name of this array'; A.length",
                                    1000,
                                    (A = new Array(1000), A.name = 'name of this array', A.length ) );


    // A has Property P, P is not length, P is an array index, and ToUint32(p) is less than the
    // value of length

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(1000); A[123] = 'hola'; A[123]",
                                    'hola',
                                    (A = new Array(1000), A[123] = 'hola', A[123] ) );

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(1000); A[123] = 'hola'; A.length",
                                    1000,
                                    (A = new Array(1000), A[123] = 'hola', A.length ) );


    /*
    for ( var i = 0X0020, TEST_STRING = "var A = new Array( " ; i < 0x00ff; i++ ) {
        TEST_STRING += "\'\\"+ String.fromCharCode( i ) +"\'";
        if ( i < 0x00FF - 1   ) {
            TEST_STRING += ",";
        } else {
            TEST_STRING += ");"
        }
    }
    */
    var TEST_STRING;
    var LENGTH = 0x00ff - 0x0020;
    var A = new Array();
    var index = 0;
    for ( var i = 0X0020; i < 0x00ff; i++ ) {
        A[index++] = String.fromCharCode( i );
    }

    array[item++] = Assert.expectEq(   
                                    TEST_STRING +" A[150] = 'hello'; A[150]",
                                    'hello',
                                    (A[150] = 'hello', A[150]) );

    array[item++] = Assert.expectEq(   
                                    TEST_STRING +" A[150] = 'hello', A.length",
                                    LENGTH,
                                    (A[150] = 'hello', A.length) );

    // A has Property P, P is not length, P is an array index, and ToUint32(p) is not less than the
    // value of length

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(); A[123] = true; A.length",
                                    124,
                                    (A = new Array(), A[123] = true, A.length ) );

    array[item++] = Assert.expectEq(   
                                    "var A = new Array(0,1,2,3,4,5,6,7,8,9,10); A[15] ='15'; A.length",
                                    16,
                                    (A = new Array(0,1,2,3,4,5,6,7,8,9,10), A[15] ='15', A.length ) );

    // !!@ As of 04jan05, the AVM+ does not like the (i <= 10) line (where '15' is a different type than void 0)
    for ( var i = 0; i < A.length; i++, item++ ) {
        var temp;
        if (i <= 10)
            temp = i;
        else if (i == 15)
            temp = '15';
        else
            temp = void 0;

        array[item] = Assert.expectEq( 
                                    "var A = new Array(0,1,2,3,4,5,6,7,8,9,10); A[15] ='15'; A[" +i +"]",
                                    temp, //(i <= 10) ? i : ( i == 15 ? '15' : void 0 ),
                                    A[i] );
    }
    // P is not an array index, and P is not "length"

    try{
        thisErr = "no error";
        var A = new Array(); A.join.length = 4; A.join.length;
    }catch(e:ReferenceError){
        thisErr = e.toString();
    } finally{
        array[item++] = Assert.expectEq(
                                    "var A = new Array(); A.join.length = 4; A.join.length",
                                    "ReferenceError: Error #1074",
                                    Utils.referenceError(thisErr));
    }

    try{
        thisErr = "no error";
        var A = new Array(); A.join.length = 4; A.length;
    }catch(e:ReferenceError){
        thisErr = e.toString();
    } finally{
        array[item++] = Assert.expectEq(
                                    "var A = new Array(); A.join.length = 4; A.length",
                                    "ReferenceError: Error #1074",
                                    Utils.referenceError(thisErr));
    }

    return array;
}
