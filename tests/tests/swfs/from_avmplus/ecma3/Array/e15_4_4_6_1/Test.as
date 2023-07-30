/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_6.as
 *  ECMA Section:       15.4.4.6 Array.prototype.pop()
 *  Description:        Test Case for pop function of Array Class.
 *          The last element is removed from the array and returned.
 *          1.  Call the [[Get]] method of this object with argument "length".
 *          2.  Call ToUint32(Result(1)).
 *          3.  If Result(2) is not zero, go to step 6.
 *          4.  Call the [[Put]] method of this object with arguments "length" and Result(2).
 *          5.  Return undefined.
 *          6.  Call ToString(Result(2)-1).
 *          7.  Call the [[Get]] method of this object with argument Result(6).
 *          8.  Call the [[Delete]] method of this object with argument Result(6).
 *          9.  Call the [[Put]] method of this object with arguments "length" and (Result(2)-1).
 *          10. Return Result(7).

 *          This test case is for checking if the length of
 *          a non initailized array after poping is zero or not.
 *          Given by Werner Sharp (wsharp@macromedia.com)

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *
 */

// var SECTION = "15.4.4.6";
// var TITLE   = "Array.pop";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;


    // Initialize a new Array Object.
    var MYARR = new Array();

    MYARR.pop()

    // Check if the length of the array is equivalent to 0 or not?
    array[item++] = Assert.expectEq(  "MYARR = []; MYARR.pop(); MYARR.length;", 0, MYARR.length );

    return ( array );

}
