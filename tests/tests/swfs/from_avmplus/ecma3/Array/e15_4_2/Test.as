/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Objects";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var arr;

    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,16)] = 'hi', arr.length",      Math.pow(2,16)+1,   (arr=new Array(),  arr[Math.pow(2,16)] = 'hi', arr.length ) );

    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,30)-2] = 'hi', arr.length",    Math.pow(2,30)-1,   (arr=new Array(),  arr[Math.pow(2,30)-2] = 'hi', arr.length ) );
    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,30)-1] = 'hi', arr.length",    Math.pow(2,30),     (arr=new Array(),  arr[Math.pow(2,30)-1] = 'hi', arr.length ) );
    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,30)] = 'hi', arr.length",      Math.pow(2,30)+1,   (arr=new Array(),  arr[Math.pow(2,30)] = 'hi', arr.length ) );

    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,31)-2] = 'hi', arr.length",    Math.pow(2,31)-1,   (arr=new Array(),  arr[Math.pow(2,31)-2] = 'hi', arr.length ) );
    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,31)-1] = 'hi', arr.length",    Math.pow(2,31),     (arr=new Array(),  arr[Math.pow(2,31)-1] = 'hi', arr.length ) );
    array[item++] = Assert.expectEq(  "var arr=new Array(),  arr[Math.pow(2,31)] = 'hi', arr.length",      Math.pow(2,31)+1,   (arr=new Array(),  arr[Math.pow(2,31)] = 'hi', arr.length ) );

    array[item++] = Assert.expectEq(  "var arr = new Array(0,1,2,3,4,5), arr.length = 2, String(arr)",     "0,1",              (arr = new Array(0,1,2,3,4,5), arr.length = 2, String(arr) ) );
    array[item++] = Assert.expectEq(  "var arr = new Array(0,1), arr.length = 3, String(arr)",             "0,1,",             (arr = new Array(0,1), arr.length = 3, String(arr) ) );
//    array[item++] = Assert.expectEq(  "var arr = new Array(0,1,2,3,4,5), delete arr[0], arr.length",       5,                  (arr = new Array(0,1,2,3,4,5), delete arr[0], arr.length ) );
//    array[item++] = Assert.expectEq(  "var arr = new Array(0,1,2,3,4,5), delete arr[6], arr.length",       5,                  (arr = new Array(0,1,2,3,4,5), delete arr[6], arr.length ) );

    return ( array );
}
