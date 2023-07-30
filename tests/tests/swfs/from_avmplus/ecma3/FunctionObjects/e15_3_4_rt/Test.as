/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.3.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of the Function Prototype Object";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var origFunctionToString = Function.prototype.toString;

    array[item++] = Assert.expectEq(   
         "var myfunc = Function.prototype; myfunc.toString = Object.prototype.toString; myfunc.toString()",
         true,
         (myfunc = Function.prototype, myfunc.toString = Object.prototype.toString, myfunc.toString()).indexOf("[object Function-")==0
         );

    array[item++] = Assert.expectEq(   "Function.prototype.valueOf",       Object.prototype.valueOf,   Function.prototype.valueOf );
    
    array[item++] = Assert.expectEq(   "Function.prototype()", undefined, Function.prototype() );

    var thisError = "no error";
    try{
        Function.prototype(1,true,false,'string', new Date(),null);
    }catch(e1:Error){
        thisError=e1.toString();
    }finally{
        array[item++] = Assert.expectEq(   "Function.prototype(1,true,false,'string', new Date(),null)","no error",Utils.typeError(thisError) );
    }
  
    //restore
    Function.prototype.toString = origFunctionToString;

    return ( array );
}
