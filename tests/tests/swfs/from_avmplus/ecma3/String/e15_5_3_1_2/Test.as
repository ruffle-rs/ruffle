/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.5.3.1-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the String Constructor";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var thisError = "no exception thrown";
    try{
        String.prototype=null;
    } catch(e) {
        thisError=e.toString()
    } finally {
        array[item++] = Assert.expectEq( 
                                "String.prototype=null;String.prototype",
                                "ReferenceError: Error #1074",
                                Utils.referenceError( thisError) );
    }
    return ( array );
}
