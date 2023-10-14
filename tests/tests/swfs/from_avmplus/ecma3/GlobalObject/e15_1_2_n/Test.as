/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.1-2-n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Global Object";


    var testcases = new Array();
    thisError="no error";
    try{
       this();
    }catch(e:TypeError){
       thisError=e.toString();
    }finally{

    testcases[0] = Assert.expectEq(   
                                    "this()",
                                    "TypeError: Error #1006",
                                    Utils.typeError(thisError) );
    }
