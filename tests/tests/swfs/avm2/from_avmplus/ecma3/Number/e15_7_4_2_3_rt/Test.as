/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.2-3-n";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array:Array = new Array();
    var item:Number= 0;
    var thisError:String = "no error thrown";
    var o:String = new String();
    
    var expectedError = 1056;
    //if (as3Enabled) {
        expectedError = 1037;
    //}
    
    try{
     
        o.toString = Number.prototype.toString;
        o.toString();
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq(
                            "o = new String(); o.toString = Number.prototype.toString; o.toString()",
                            Utils.REFERENCEERROR+expectedError,
                            Utils.referenceError(thisError));
                     
    }
     
    //o = new String();
    //o.toString = Number.prototype.toString;
    
    //array[item++] = Assert.expectEq(  "o = new String(); o.toString = //Number.prototype.toString; o.toString()",  "error",    o.toString() );

    return ( array );
}
