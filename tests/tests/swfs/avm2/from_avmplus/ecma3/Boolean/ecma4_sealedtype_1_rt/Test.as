/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
    In Ecma4 there are three sealed types; Boolean, Number and String
    You cannot set properties of an instance of a sealed type

    Should throw a ReferenceError

*/
//     var SECTION = "ECMA_4";
//     var VERSION = "ECMA_4";
//     var TITLE   = "tostr=Boolean.prototype.toString;x=new Boolean();x.toString=tostr;";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError="no error";

    try {
        tostr=Boolean.prototype.toString;
        x=new Boolean();
        x.toString=tostr;
    } catch (e) {
        thisError = e.toString();
    } finally {
        
        //TO-DO: Remove as3Enabled
      // if (as3Enabled) {
            array[item++] =Assert.expectEq(
                                        "Cannot assign to a method toString on Boolean.",
                                        "ReferenceError: Error #1037",
                                        Utils.referenceError( thisError ) );
        /*} else {
            array[item++] =Assert.expectEq(
                                        "Cannot create a property on Boolean",
                                        "ReferenceError: Error #1056",
                                        Utils.referenceError( thisError ) );
        }*/
    }
    return ( array );
}
