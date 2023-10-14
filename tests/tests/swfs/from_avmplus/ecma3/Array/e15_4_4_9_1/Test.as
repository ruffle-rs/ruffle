/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_9.as
 *  ECMA Section:       15.4.4.9 Array.prototype.shift()
 *  Description:        Test Case for reverse function of Array Class.
 *          The first element of the array is removed from the array and returned.
 *
 *          1. Call the [[Get]] method of this object with argument "length".
 *          2. Call ToUint32(Result(1)).
 *          3. If Result(2) is not zero, go to step 6.
 *          4. Call the [[Put]] method of this object with arguments "length" and Result(2).
 *          5. Return undefined.
 *          6. Call the [[Get]] method of this object with argument 0.
 *          7. Let k be 1.
 *          8. If k equals Result(2), go to step 18.
 *          9. Call ToString(k).
 *          10. Call ToString(k-1).
 *          11. If this object has a property named by Result(9), go to step 12; but if this object has no property
 *              named by Result(9), then go to step 15.
 *          12. Call the [[Get]] method of this object with argument Result(9).
 *          13. Call the [[Put]] method of this object with arguments Result(10) and Result(12).
 *          14. Go to step 16.
 *          15. Call the [[Delete]] method of this object with argument Result(10).
 *          16. Increase k by 1.
 *          17. Go to step 8.
 *          18. Call the [[Delete]] method of this object with argument ToString(Result(2)-1).
 *          19. Call the [[Put]] method of this object with arguments "length" and (Result(2)-1).
 *          20. Return Result(6).
 *
 *  Note:
 *  The shift function is intentionally generic; it does not require that its this value be an Array object.
 *  Therefore it can be transferred to other kinds of objects for use as a method. Whether the shift
 *  function can be applied successfully to a host object is implementation-dependent.

 *          This test case is for checking if the length of
 *          a non initailized array after shifting; is zero or not.
 *          Given by Werner Sharp (wsharp@macromedia.com)

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *
 */

// var SECTION = "15.4.4.9";
// var TITLE   = "Array.shift";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // Create an array from which we will shift an element.
    var MYARR = new Array();

    // As the array is not initialized with any elements, we should get a value of zero
    // for the length of the array.
    MYARR.shift();


    array[item++] = Assert.expectEq(  "MYARR = []; MYARR.shift(); MYARR.length", 0, MYARR.length );


    return ( array );

}
