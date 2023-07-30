/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
    In Ecma4 there are three sealed types; Boolean, Number and String
    You cannot set properties of an instance of a sealed type

    Should throw a ReferenceError

*/
//     var SECTION = "ECMA_4";
//     var VERSION = "ECMA_4";
//     var TITLE   = "tostr=Number.prototype.toString;x=new Number(4);x.toString=tostr;";

    var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;

    var thisError:String = "no Exception thrown";
    var tostr=Number.prototype.toString;
    var x:Number=new Number(4);
    try{

        x.toString=tostr;
        x.toString();
    } catch(e:ReferenceError){
        thisError = e.toString();
    } finally {
        
        //TO-DO: commenting as3Enabled
        var expectedError=1037;
        /*var expectedError = 1056;
        if (as3Enabled) {
            expectedError = 1037;
        }*/
        array[item] = Assert.expectEq( "toStr=Number.prototype.toString;x=new Number(4);x.toString=tostr",
                     Utils.REFERENCEERROR+expectedError,
                     Utils.referenceError(thisError) );
    }
    return ( array );
}
