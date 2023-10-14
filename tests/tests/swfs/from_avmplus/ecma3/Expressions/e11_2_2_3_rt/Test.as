/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "11_2_2_3_n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var NULL = null;

    try{
       var o = new NULL();
    }catch(e:Error){
       thisError = e.toString();
    }finally{
       array[item++] = Assert.expectEq( 
                                    "var o = new NULL()",
                                    "TypeError: Error #1007",
                                    Utils.typeError(thisError) );
     }

    /*array[item++] = Assert.expectEq( 
                                    "NULL = null; var o = new NULL()",
                                    "error",
                                    o = new NULL() );*/
    return array;
}

function TestFunction() {
    return arguments;
}
