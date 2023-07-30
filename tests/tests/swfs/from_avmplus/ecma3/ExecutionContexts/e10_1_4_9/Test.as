/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "10.1.4-9";
//     var VERSION = "ECMA_2";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // array[item++] = Assert.expectEq(  "with MyObject, NEW_PROPERTY", "", "" );

   // TODO: REVIEW AS4 CONVERSION ISSUE  


  

        var MYOBJECT = new MyObject();
        var RESULT   = "hello";
        var myResult = new Object();

        with ( MYOBJECT ) {
            NEW_PROPERTY = RESULT;
        }
        myResult.actual = NEW_PROPERTY;
        myResult.expect = RESULT;

        Assert.expectEq(  "with MyObject, NEW_PROPERTY", myResult.expect, myResult.actual);

    return ( array );
}
function MyObject( n ) {
    // cn:  __proto__ not ecma3 compliant
    //this.__proto__ = Number.prototype;
}
