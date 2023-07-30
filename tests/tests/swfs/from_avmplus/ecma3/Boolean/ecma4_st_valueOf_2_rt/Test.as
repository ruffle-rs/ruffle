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
//     var TITLE   = "valof=Boolean.prototype.valueOf;booleanObj=false;booleanObj.valueOf=valof;";

    var testcases = getTestCases();

function getTestCases() {
    var booleanObj:Boolean = new Boolean();

    var thisError:String = "no error thrown";
    var array:Array = new Array();
    var item:Number = 0;
    var valof=Boolean.prototype.valueOf;
    try{
        booleanObj.ValueOf=false;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++] =Assert.expectEq("Cannot create a property on Boolean","ReferenceError: Error #1056",Utils.referenceError( thisError ) );
    }

    try{
        booleanObj.ValueOf=valof;
        booleanObj.ValueOf();
    }catch(e1:ReferenceError){
        thisError=e1.toString();
    }finally{
        array[item++] =Assert.expectEq("Cannot create a property on Boolean","ReferenceError: Error #1056",Utils.referenceError( thisError ) );
    }
    return array;
}
