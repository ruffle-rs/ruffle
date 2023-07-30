/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;
    var o:String = new String();
    var thisError:String = "no exception thrown";
    try{

        o.toString = Number.prototype.toString;
    } catch(e:ReferenceError) {
        thisError = e.toString();
    } finally {
        var expectedError=1037;
        /*var expectedError = 1056;
        if (as3Enabled) {
            expectedError = 1037;
        }*/
        array[item++] = Assert.expectEq(  "o = new String(); o.toString = Number.prototype.toString; o.toLocaleString()",
                                            Utils.REFERENCEERROR+expectedError,
                                            Utils.referenceError(thisError) );
    }
    return ( array );
}
