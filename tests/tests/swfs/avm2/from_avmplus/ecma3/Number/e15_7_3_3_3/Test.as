/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.3.3-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.MIN_VALUE:  ReadOnly Attribute";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError:String="no error";
    try{
        Number.MIN_VALUE=0;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Trying to verify the ReadOnly attribute of Number.MIN_VALUE","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(
                    // TO-DO: COMMENTING SECTION 
                    ////SECTION,
                    "Number.MIN_VALUE=0; Number.MIN_VALUE",
                    Number.MIN_VALUE,
                    Number.MIN_VALUE );
    return ( array );
}
