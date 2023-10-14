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

 *  Notes:
 *  The pop function is intentionally generic; it does not require that its 'this' value
 *  be an Array object.
 *  Therefore it can be transferred to other kinds of objects for use as a method.
 *  Whether the pop function can be applied successfully to a host object is implementation-dependent.

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *  Modified By:        Subha Subramanian
 *  Date:               01/04/2006(Added test cases for calling pop() method on an array  *  that is empty and test case for transferring pop method to other object which is not an  *  array
 *
 */

// var SECTION = "15.4.4.6";
// var TITLE   = "Array.pop";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

        var MYEMPTYARRAY = new Array();
        array[item++] = Assert.expectEq(  "MYEMPTYARRAY = new Array(); MYEMPTYARRAY.pop();", undefined, MYEMPTYARRAY.pop());

    // Create an array from which we will pop an element.
    var MYARR = new Array( 2, 1, 8, 6 );
    var EXPARR = [ 2, 1, 8 ];


    var EXP_RESULT = MYARR.pop();

    array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.pop();", 6, EXP_RESULT );

    for (var MYVAR = 0; ( MYVAR < MYARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.pop();", EXPARR[MYVAR], MYARR[MYVAR] );
    }

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    // COMMENT OUT WHOLE BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    /* 
    if (!as3Enabled) {
        //pop method is generic so can be transferred to other types of objects
        var obj = new Object();
        obj.pop = Array.prototype.pop;
        obj.length = 4;
        obj[0] = 2;
        obj[1] = 1;
        obj[2] = 8;
        obj[3] = 6;
        var EXP_OBJRESULT = obj.pop();
    
        array[item++] = Assert.expectEq(  "obj.pop()", 6, EXP_OBJRESULT );
    
        for (var MYVAR1 = 0; ( MYVAR1 < obj.length ); MYVAR1++)
        {
            array[item++] = Assert.expectEq(  "obj.pop()", EXPARR[MYVAR1], obj[MYVAR1] );
        }
    }
    */



    return ( array );

}
