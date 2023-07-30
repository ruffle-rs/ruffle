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

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *  Modified By:        Subha Subramanian
 *  Date:               01/04/2006
 *  Added test cases to test shift function called on an empty array and test cases to test  *  shift function on other objects which are not array objects
 */

// var SECTION = "15.4.4.9";
// var TITLE   = "Array.shift";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

        var MYEMPTYARR = new Array();

        array[item++] = Assert.expectEq(  "MYEMPTYARR = []; MYEMPTYARR.shift();", undefined, MYEMPTYARR.shift() );


    // Create an array from which we will shift an element.
    var MYARR = new Array( 2, 1, 8, 6 );
    var EXPARR = [ 1, 8, 6 ];

    array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.shift();", 2, MYARR.shift() );



    for (var MYVAR = 0; ( MYVAR < EXPARR.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.shift();", EXPARR[MYVAR], MYARR[MYVAR] );
    }


    array[item++] = Assert.expectEq(  "MYARR = [2,1,8,6]; MYARR.shift();MYARR.length", 3, MYARR.length );


        var MYBIGARR = []

        for (var i = 0; i<101; i++){
            MYBIGARR[MYBIGARR.length] = i;
       }

       array[item++] = Assert.expectEq(  "MYBIGARR = [0,1,2,.....,100]; MYBIGARR.shift();", 0, MYBIGARR.shift() );

       array[item++] = Assert.expectEq(  "MYBIGARR = [0,1,2,.....,100]; MYBIGARR.shift();MYBIGARR.length", 100, MYBIGARR.length );

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    // COMMENT OUT WHOLE BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    /* 
    if (!as3Enabled) {
       //shift method is not generic it can transferred to other objects for use as method
        var obj = new Object();
        obj.shift = Array.prototype.shift;
        obj.length = 4;
        obj[0] = 0;
        obj[1] = 1;
        obj[2] = 2;
        obj[3] = 3;

        array[item++] = Assert.expectEq(  "obj = new Object(); obj.shift()", 0, obj.shift() );

        array[item++] = Assert.expectEq(  "obj = new Object(); obj.shift();obj.length", 3, obj.length );
    }
    */
    
    return ( array );

}
