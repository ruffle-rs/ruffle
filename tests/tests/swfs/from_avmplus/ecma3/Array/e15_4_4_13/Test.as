/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_13.as
 *  ECMA Section:       15.4.4.13 Array.unshift()
 *  Description:        Test Case for unshift function of Array Class.
 *          The arguments given to unshift are prepended to the start
 *          of the array, such that their order within the array is the
 *          same as the order in which they appear in the argument list.

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *  Modified By:        Subha Subramanian
 *  Date:               01/05/2006
 *  Details:            Added test cases to test the length of the unshift method, unshift    *                      without any parameters and to test that the unshift can be
 *                      transferred to other objects for use as method
 */

// var SECTION = "15.4.4.13";
// var TITLE   = "Array.unshift";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
        var myobj = new Object();
    var item = 0;

       array[item++] = Assert.expectEq(  "Array.prototype.unshift.length;",0, Array.prototype.unshift.length);

        var MYEMPTYARR = new Array();

        array[item++] = Assert.expectEq(  "MYEMPTYARR = new Array();MYEMPTYARR.unshift();", 0, MYEMPTYARR.unshift() );
        array[item++] = Assert.expectEq(  "MYEMPTYARR = new Array();MYEMPTYARR.unshift(0);", 1, MYEMPTYARR.unshift(0) );


        var MYARR = new Array();
    MYARR.push( 1, 2 );


       var MYVAR:Number = 3

    array[item++] = Assert.expectEq(  "MYARR = new Array(); MYARR.push(1, 2); MYARR.unshift(0);", MYVAR, MYARR.unshift(0) );

        for (var i = 0;i<MYARR.length; i++){

        array[item++] = Assert.expectEq(  "MYARR = new Array(); MYARR.push(1, 2); MYARR.unshift(0);MYARR", i, MYARR[i] );

        }
        //unshift method can be transferred to other objects for use as method

        myobj.unshift= Array.prototype.unshift;
        myobj.length = 3
        myobj[0] = 3;
        myobj[1] = 4;
        myobj[2] = 5;

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    // COMMENT OUT WHOLE BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    /* 
        if (!as3Enabled) {
            array[item++] = Assert.expectEq(  "myobj = new Object(); myobj.unshift= Array.prototype.unshift;myobj.unshift(0,1,2);", 6, myobj.unshift(0,1,2) );

            for (var i=0; i<6; i++){
                array[item++] = Assert.expectEq(  "myobj = new Object(); myobj.unshift= Array.prototype.unshift; myobj.unshift(0,1,2);",i, myobj[i] );
            }
        }
        */
        
    return ( array );

}
