/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "e11_2_2_7_n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var STRING = new String("hi");
    try{
       var s = new STRING();
    }catch(e:Error){
       thisError = e.toString();
    }finally{
       array[item++] = Assert.expectEq( 
                                    "var s = new STRING()",
                                    "TypeError: Error #1007",
                                    Utils.typeError(thisError) );
     }


   /* array[item++] = Assert.expectEq( 
                                    "var STRING = new String('hi'); var s = new STRING()",
                                    "error",
                                    s = new STRING() );*/
    return array;
}

function TestFunction() {
    return arguments;
}
