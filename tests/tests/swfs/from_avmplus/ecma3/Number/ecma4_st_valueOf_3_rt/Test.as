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

    Author:     mtilburg@macromedia.com
    Date:       October 13, 2004

*/
//     var SECTION = "ECMA_4";
//     var VERSION = "ECMA_4";
//     var TITLE   = "valof=Number.prototype.valueOf;x=4;x.valueOf=valof;";

    var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;

    var thisError:String = "no exception thrown";
    var valof=Number.prototype.valueOf;
    var x:Number=4;
    try{
        
        x.valueOf=valof;
        x.valueOf();
    } catch (e:ReferenceError) {
        thisError = e.toString();
    } finally {
        var expectedError=1037;
       /* var expectedError = 1056;
        if (as3Enabled) {
            expectedError = 1037;
        }*/
        array[item] = Assert.expectEq( "valof=Number.prototype.valueOf;x=4;x.valueOf=valof",
                     Utils.REFERENCEERROR+expectedError,
                     Utils.referenceError(thisError) );
    }
    return ( array );
}
