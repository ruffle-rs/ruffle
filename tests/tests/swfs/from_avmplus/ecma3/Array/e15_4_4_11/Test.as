/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_11.as
 *  ECMA Section:       15.4.4.11 Array.prototype.sort()
 *  Description:        Test Case for sort function of Array Class.
 *          The elements of the array are sorted.
 *          The sort is not necessary stable (this is, elements
 *          that compare equal do not necessarily remain in their
 *          original order.

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *
 */

// var SECTION = "15.4.4.11";
// var TITLE   = "Array.sort";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // Check for only numeric values.
    var MYARR = new Array( 2, 1, 8, 6 );
    var EXPARR = [1,2,6,8];

    MYARR.sort()

    for (var MYVAR = 0; ( MYVAR < MYARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.sort()", EXPARR[MYVAR], MYARR[MYVAR] );
    }


    // Check for only alpha-numeric values.
    var MYARR = new Array( 'a', 'd', 'Z', 'f', 'M' );
    var EXPARR = ['M', 'Z', 'a', 'd', 'f'];

    MYARR.sort()

    for (var MYVAR = 0; ( MYVAR < MYARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = ['a','d','Z','f','M']; MYARR.sort()", EXPARR[MYVAR], MYARR[MYVAR] );
    }


    // Check for numeric and alpha-numeric values.
    var MYARR = new Array( 2, 1, 'M', 'y', 'X', 66, 104 );
    var EXPARR = [1, 104, 2, 66, 'M', 'X', 'y'];

    MYARR.sort()

    for (var MYVAR = 0; ( MYVAR < MYARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = [2, 1, 'M', 'X', 'y', 66, 104]; MYARR.sort()", EXPARR[MYVAR], MYARR[MYVAR] );
    }


    return ( array );

}
