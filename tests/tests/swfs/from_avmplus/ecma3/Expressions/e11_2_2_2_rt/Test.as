/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "e11_2_2_2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    var UNDEFINED = void 0;
    try{
       var o = new UNDEFINED();
    }catch(e:Error){
       thisError = e.toString();
    }finally{
       array[item++] = Assert.expectEq( 
                                    "var o = new UNDEFINED()",
                                    "TypeError: Error #1007",
                                    Utils.typeError(thisError) );
     }


  /*array[item++] = Assert.expectEq( 
                                    "UNDEFINED = void 0; var o = new UNDEFINED()",
                                    "error",
                                    o = new UNDEFINED() );*/
    return array;
}

function TestFunction() {
    return arguments;
}
