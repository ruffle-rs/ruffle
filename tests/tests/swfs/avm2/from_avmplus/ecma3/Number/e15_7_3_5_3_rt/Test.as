/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.7.3.5-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.NEGATIVE_INFINITY";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError:String="no error";
    try{
        Number.NEGATIVE_INFINITY=0;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Trying to verify the ReadOnly attribute of Number.NEGATIVE_INFINITY","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(
                    //SECTION,
                    "Number.NEGATIVE_INFINITY=0; Number.NEGATIVE_INFINITY",
                    -Infinity,
                    Number.NEGATIVE_INFINITY );
    return ( array );
}
