/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.3.5.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Function.arguments";


    var testcases = getTestCases();

function getTestCases() {
    var array =new Array();
    var item = 0;

    thisError="no error";
    try{
        var MYFUNCTION = new Function( "return this.arguments" );
    }catch(e){
        thisError=(e.toString()).substring(0,22);
    }finally{
        array[item++] = Assert.expectEq( "Function('function body') not supported","EvalError: Error #1066",thisError);
    }

    //arguments not supported anymore
    //var MYFUNCTION = new Function();

    //array[item++] = Assert.expectEq(   "var MYFUNCTION = new Function(); MYFUNCTION.arguments",  null,   MYFUNCTION.arguments );

    return array;
}
