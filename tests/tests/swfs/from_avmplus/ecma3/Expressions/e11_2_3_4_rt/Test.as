/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "e11_2_3_4_n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Function Calls";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    try{
       null.valueOf();
    }catch(e:Error){
       thisError = e.toString();
    }finally{
       array[item++] = Assert.expectEq( 
                                    "null.valueOf()",
                                    "TypeError: Error #1009",
                                    Utils.typeError(thisError) );
     }
    return array;
}
    
