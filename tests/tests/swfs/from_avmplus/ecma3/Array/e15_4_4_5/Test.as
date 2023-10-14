/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.4.5";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype.join";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "Array.prototype.join.length",  1,  Array.prototype.join.length );

    array[item++] = Assert.expectEq(   "(new Array()).join()",     "",     (new Array()).join() );
    array[item++] = Assert.expectEq(   "(new Array(2)).join()",    ",",    (new Array(2)).join() );
    array[item++] = Assert.expectEq(   "(new Array(0,1)).join()",  "0,1",  (new Array(0,1)).join() );
    array[item++] = Assert.expectEq(   "(new Array(0,1)).join(separator)",  "0/1",  (new Array(0,1)).join("/") );
    array[item++] = Assert.expectEq(   "(new Array( Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)).join()",  "NaN,Infinity,-Infinity",   (new Array( Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY)).join() );
    array[item++] = Assert.expectEq(   "(new Array( Boolean(1), Boolean(0))).join()",   "true,false",   (new Array(Boolean(1),Boolean(0))).join() );
    array[item++] = Assert.expectEq(   "(new Array(void 0,null)).join()",    ",",    (new Array(void 0,null)).join() );

    var EXPECT_STRING = "";
    var MYARR = new Array();

    for ( var i = -50; i < 50; i+= 0.25 ) {
        MYARR[MYARR.length] = i;
        EXPECT_STRING += i +"/";
    }

    EXPECT_STRING = EXPECT_STRING.substring( 0, EXPECT_STRING.length -1 );

    array[item++] = Assert.expectEq(  "MYARR.join(separator)",  EXPECT_STRING,  MYARR.join("/") );

   var MYARR2 = [0,1,2,3,4,5,6,7,8,9]

   array[item++] = Assert.expectEq(  "MYARR2.join(separator)",  "0separator1separator2separator3separator4separator5separator6separator7separator8separator9",  MYARR2.join("separator") );

   var MYARRARR = [new Array(1,2,3),new Array(4,5,6)]

   array[item++] = Assert.expectEq(  "MYARRARR.join(separator)",  "1,2,34,5,6",MYARRARR.join("") );

   var obj;
   var MYUNDEFARR = [obj];

   array[item++] = Assert.expectEq(  "MYUNDEFARR.join()",  "",MYUNDEFARR.join() );

   var MYNULLARR = [null]

   array[item++] = Assert.expectEq(  "MYNULLARR.join()",  "",MYNULLARR.join());

   var MYNULLARR2 = new Array(null);

   array[item++] = Assert.expectEq(  "MYNULLARR2.join()",  "",MYNULLARR2.join() );

   var MyAllArray = new Array(new String('string'),new Array(1,2,3),new Number(100000),Boolean(0),Number.MAX_VALUE)

   array[item++] = Assert.expectEq(  "MyAllArray.join(separator)",  "string&1,2,3&100000&false&1.79769313486231e+308",MyAllArray.join("&") );



   return ( array );
}
