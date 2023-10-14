/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_12.as
 *  ECMA Section:       15.4.4.10 Array.prototype.splice()
 *  Description:        Test Case for push function of Array Class.
 *          The method takes 2 arguments - start and end.
 *          It returns an array containing the elements of
 *          the array from element start upto but not including
 *          the element end.
 *          If end is undefined then end is the last element.
 *          If start is negative, it is treated as (length+start)
 *          where length is the length of the array.

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *
 */

// var SECTION = "15.4.4.12";
// var TITLE   = "Array.splice";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var MYARR = new Array();
    MYARR = [0, 2, 3, 4, 5];

    var RESULTARR = MYARR.splice(1);

    var EXPCTARR = [ 2, 3, 4, 5 ]

    for (var MYVAR = 0; ( MYVAR < RESULTARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = new Array( 0, 2, 3, 4, 5 ); MYARR.splice(1);", EXPCTARR[MYVAR], RESULTARR[MYVAR] );
    }


    return ( array );

}
