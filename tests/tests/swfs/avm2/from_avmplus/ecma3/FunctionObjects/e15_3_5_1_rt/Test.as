/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.3.5.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Function.length";
    var BUGNUMBER="104204";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError="no error";
    try{
        var f = new Function( "a","b", "c", "return f.length");
    }catch(e:Error){
        thisError=(e.toString()).substring(0,22);
    }finally{
        array[item++] = Assert.expectEq( "Function('function body') not supported","EvalError: Error #1066",thisError);
    }

    var f = new Function();
    
    array[item++] = Assert.expectEq( 
        'var f = new Function(); f()',
        undefined,
        f() );


    array[item++] = Assert.expectEq( 
        'var f = new Function(); f(1,2,3,4,5)',
        undefined,
        f(1,2,3,4,5) );
        
    return array;
}
