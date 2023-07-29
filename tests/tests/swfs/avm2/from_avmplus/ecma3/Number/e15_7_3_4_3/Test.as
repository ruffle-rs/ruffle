/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.7.3.4-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.NaN";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError = "no error";
    try{
        Number.NaN=0;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Trying to verify the ReadOnly attribute of Number.NaN","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(
                    ////SECTION,
                    "Number.NaN=0; Number.NaN",
                    Number.NaN,
                    Number.NaN );

    return ( array );
}
