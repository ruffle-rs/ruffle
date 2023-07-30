/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.3.1.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Function Constructor Called as a Function";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError="no error";
    try{
        var MyObject = Function( "value", "this.value = value; this.valueOf =  Function( 'return this.value' ); this.toString =  Function( 'return String(this.value);' )" );
    }catch(e1:EvalError){
        thisError=(e1.toString()).substring(0,22);
    }finally{
        array[item++] = Assert.expectEq(   "Function('function body') is not supported","EvalError: Error #1066",thisError);
    }
    var myfunc:Function = Function();
    myfunc.myToString=Object.prototype.toString;

    array[item++] = Assert.expectEq(   
                                    "myfunc = Function(); myfunc.myToString = Object.prototype.toString; myfunc.myToString()",
                                    true,
                                    myfunc.myToString().indexOf("[object Function-") == 0
                                     );
    thisError="no error";
    try{
        myfunc.toString = Object.prototype.toString;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }
    array[item++] = Assert.expectEq(   
                                    "myfunc = Function(); myfunc.toString = Object.prototype.toString; myfunc.toString()",
                                    "no error",
                                    Utils.referenceError(thisError) );

    array[item++] = Assert.expectEq(   "myfunc.length",0,                     myfunc.length );
    array[item++] = Assert.expectEq(   "myfunc.prototype.toString()",              "[object Object]",      myfunc.prototype.toString() );
    array[item++] = Assert.expectEq(   "myfunc.prototype.constructor",             myfunc,                 myfunc.prototype.constructor );
  //array[item++] = Assert.expectEq(   "myfunc.arguments",                         undefined,myfunc.arguments );


    return ( array );
}
