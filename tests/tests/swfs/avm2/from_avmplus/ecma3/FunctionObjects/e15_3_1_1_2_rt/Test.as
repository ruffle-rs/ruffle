/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.3.1.1-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The Function Constructor Called as a Function";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item:Number= 0;
    var thisError:String="no error";
    
    try{
        var myfunc1 =  Function("a","b","c", "return a+b+c" );
    }catch(e:Error){
        thisError=(e.toString()).substring(0,22);
    }finally{
        array[item++] = Assert.expectEq( "Function('function body') not supported","EvalError: Error #1066",thisError);
    }

    return ( array );
}
