/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          e15_4_4_7.as
 *  ECMA Section:       15.4.4.7 Array.prototype.push()
 *  Description:        Test Case for push function of Array Class.
 *          The arguments are appended to the end of the array,
 *          in the order in which they appear. The new length of
 *          the array is returned as a result of the call.

 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *
 */

// var SECTION = "15.4.4.7";
// var TITLE   = "Array.push";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

        var MYEMPTYARR:Array= new Array();

        var EMPTYARRLENGTH = 0;

        array[item++] = Assert.expectEq(  "MYEMPTYARR = new Array(); MYEMPTYARR.push();", EMPTYARRLENGTH, MYEMPTYARR.push() );

        var MYEMPTYARRLENGTHAFTERPUSH = 1;

        array[item++] = Assert.expectEq(  "MYEMPTYARR = new Array(); MYEMPTYARR.push(2);", MYEMPTYARRLENGTHAFTERPUSH, MYEMPTYARR.push(2) );

        array[item++] = Assert.expectEq(  "MYEMPTYARR = new Array(); MYEMPTYARR[0];", 2, MYEMPTYARR[0] );


        var MYARR = new Array( 1, 4 );

    var MYVAR = 2;
    var ARRLENGTH = 3;

        array[item++] = Assert.expectEq(  "MYARR = new Array(); MYARR.push();", 2, MYARR.push() );


    array[item++] = Assert.expectEq(  "MYARR = new Array(); MYARR.push(2);", ARRLENGTH, MYARR.push(MYVAR) );

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    // COMMENT OUT WHOLE BLOCK SINCE IT SHOULD ONLY RUN IF NOT as3Enabled (i.e. pre AS3)
    /* 
    if (!as3Enabled) {
        //push function is intentionally generic.  It does not require its this value to be         //an array object

        var obj = new Object();
        obj.push = Array.prototype.push;
        obj.length = 4
        obj[0]=0;
        obj[1]=1;
        obj[2]=2;
        obj[3]=3;

        array[item++] = Assert.expectEq(  "var obj = new Object(); obj.push(4);", 5, obj.push(4) );
    }
    */
    
    var MYBIGARR = []

    for (var i=0;i<101;i++){
        MYBIGARR[MYBIGARR.length] = i;
    }

    array[item++] = Assert.expectEq(  "var MYBIGARR[i] = i; MYBIGARR.push(101);", 102, MYBIGARR.push(101) );

    return ( array );

}
