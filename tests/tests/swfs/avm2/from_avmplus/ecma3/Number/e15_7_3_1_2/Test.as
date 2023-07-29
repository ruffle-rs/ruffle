/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;


//     var SECTION = "15.7.3.1-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Number.prototype";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError:String="no error";
    var NUM_PROT = Number.prototype
    try{
       Number.prototype = null;
    }catch(e:ReferenceError){
       thisError=e.toString();
    }finally{
       array[item++]= Assert.expectEq("Trying to verify the ReadOnly attribute of Number.prototype","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }

    array[item++] = Assert.expectEq(   
                                    "var NUM_PROT = Number.prototype; Number.prototype = null; Number.prototype == NUM_PROT",
                                    true,
                                    (Number.prototype == NUM_PROT ) );
    
    
    try{
        Number.prototype=0;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]= Assert.expectEq("Trying to verify the ReadOnly property of Number.prototype","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(   
                                    "Number.prototype=0; Number.prototype",
                                    Number.prototype,
                                    Number.prototype );

    return ( array );
}
