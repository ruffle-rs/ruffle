/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.4.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype.toLocaleString";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "Array.prototype.toLocaleString.length",  0,  Array.prototype.toLocaleString.length );

    array[item++] = Assert.expectEq(   "(new Array()).toLocaleString()",     "",     (new Array()).toLocaleString() );
    array[item++] = Assert.expectEq(   "(new Array(2)).toLocaleString()",    ",",    (new Array(2)).toLocaleString() );
    array[item++] = Assert.expectEq(   "(new Array(0,1)).toLocaleString()",  "0,1",  (new Array(0,1)).toLocaleString() );
    array[item++] = Assert.expectEq(   "(new Array( Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)).toLocaleString()",  "NaN,Infinity,-Infinity",   (new Array( Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)).toLocaleString() );
    array[item++] = Assert.expectEq(   "(new Array(Boolean(1), Boolean(0))).toLocaleString()", "[object Boolean],[object Boolean]" ,   (new Array(Boolean(1), Boolean(0))).toLocaleString());
    array[item++] = Assert.expectEq(   "(new Array(void 0,null)).toLocaleString()",    ",",                    (new Array(void 0,null)).toLocaleString() );

    var EXPECT_STRING = "";
    var MYARR = new Array();

    for ( var i = -50; i < 50; i+= 0.25 ) {
        MYARR[MYARR.length] = i;
        EXPECT_STRING += i +",";
    }

    EXPECT_STRING = EXPECT_STRING.substring( 0, EXPECT_STRING.length -1 );

    array[item++] = Assert.expectEq(  "MYARR.toLocaleString()",  EXPECT_STRING,  MYARR.toLocaleString() );

    var MYARR2 = [0,1,2,3,4,5,6,7,8,9]

   array[item++] = Assert.expectEq(  "MYARR2.toLocaleString()",  "0,1,2,3,4,5,6,7,8,9",  MYARR2.toLocaleString() );


   var MYARRARR = [new Array(1,2,3),new Array(4,5,6)]

   array[item++] = Assert.expectEq(  "MYARRARR.toLocaleString()",  "1,2,3,4,5,6",MYARRARR.toLocaleString() );

   var obj;
   var MYUNDEFARR = [obj];

   array[item++] = Assert.expectEq(  "MYUNDEFARR.toLocaleString()",  "",MYUNDEFARR.toLocaleString() );

   var MYNULLARR = [null]

   array[item++] = Assert.expectEq(  "MYNULLARR.toLocaleString()",  "",MYNULLARR.toLocaleString() );

   var MYNULLARR2 = new Array(null);

   array[item++] = Assert.expectEq(  "MYNULLARR2.toLocaleString()",  "",MYNULLARR2.toLocaleString() );

   var MyAllArray = new Array(new String('string'),new Array(1,2,3),new Number(100000),Boolean(0),Number.MAX_VALUE)

   array[item++] = Assert.expectEq(  "MyAllArray.toLocaleString()",  "[object String],1,2,3,100000,[object Boolean],1.79769313486231e+308",MyAllArray.toLocaleString() );



    return ( array );
}
