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
//     var TITLE   = "Boolean.myString = Boolean.prototype.toString;";

    var testcases = getTestCases();

function getTestCases() {
    var array:Array = new Array();
    var item:Number = 0;
    var thisError:String="no error";

    try{
        Boolean.myString = Boolean.prototype.toString;
    } catch (e:ReferenceError) {
        thisError = e.toString();
    } finally{
        array[item++] =Assert.expectEq("Cannot create a property on Boolean","no error",Utils.referenceError( thisError ) );
    }
    return ( array );
}
