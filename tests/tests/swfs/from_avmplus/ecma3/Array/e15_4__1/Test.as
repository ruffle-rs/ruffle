/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.4-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Objects";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var myarr;

    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,32)-2]='hi'; myarr[Math.pow(2,32)-2]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,32)-2]='hi', myarr[Math.pow(2,32)-2] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,32)-2]='hi'; myarr.length",
                                    (Math.pow(2,32)-1),
                                    (myarr = new Array(), myarr[Math.pow(2,32)-2]='hi', myarr.length )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,32)-3]='hi'; myarr[Math.pow(2,32)-3]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,32)-3]='hi', myarr[Math.pow(2,32)-3] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,32)-3]='hi'; myarr.length",
                                    (Math.pow(2,32)-2),
                                    (myarr = new Array(), myarr[Math.pow(2,32)-3]='hi', myarr.length )
                                );

    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)-2]='hi'; myarr[Math.pow(2,31)-2]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,31)-2]='hi', myarr[Math.pow(2,31)-2] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)-2]='hi'; myarr.length",
                                    (Math.pow(2,31)-1),
                                    (myarr = new Array(), myarr[Math.pow(2,31)-2]='hi', myarr.length )
                                );

    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)-1]='hi'; myarr[Math.pow(2,31)-1]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,31)-1]='hi', myarr[Math.pow(2,31)-1] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)-1]='hi'; myarr.length",
                                    (Math.pow(2,31)),
                                    (myarr = new Array(), myarr[Math.pow(2,31)-1]='hi', myarr.length )
                                );


    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)]='hi'; myarr[Math.pow(2,31)]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,31)]='hi', myarr[Math.pow(2,31)] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,31)]='hi'; myarr.length",
                                    (Math.pow(2,31)+1),
                                    (myarr = new Array(), myarr[Math.pow(2,31)]='hi', myarr.length )
                                );

    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,30)-2]='hi'; myarr[Math.pow(2,30)-2]",
                                    "hi",
                                    (myarr = new Array(), myarr[Math.pow(2,30)-2]='hi', myarr[Math.pow(2,30)-2] )
                                );
    array[item++] = Assert.expectEq(   
                                    "var myarr = new Array(); myarr[Math.pow(2,30)-2]='hi'; myarr.length",
                                    (Math.pow(2,30)-1),
                                    (myarr = new Array(), myarr[Math.pow(2,30)-2]='hi', myarr.length )
                                );
    return ( array );
}
