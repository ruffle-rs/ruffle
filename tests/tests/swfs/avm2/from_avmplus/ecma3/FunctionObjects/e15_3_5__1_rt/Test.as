/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.3.5-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of Function Instances";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var args = "";

    for ( var i = 0; i < 2000; i++ ) {
        args += "arg"+i;
        if ( i != 1999 ) {
            args += ",";
        }
    }

    var s = "";

    for ( var i = 0; i < 2000; i++ ) {
        s += ".0005";
        if ( i != 1999 ) {
            s += ",";
        }
    }

    thisError="no error";
    try{
        MyFunc = new  Function( args, "var r=0; for (var i = 0; i < MyFunc.length; i++ ){ if ( eval('arg'+i) == void 0) break; else r += eval('arg'+i); }; return r");
    }
    catch(e1:EvalError){
        thisError=(e1.toString()).substring(0,22);
    }
    finally{
        //print(e.toString());
        array[item++] = Assert.expectEq( "Function('function body') not supported","EvalError: Error #1066",thisError);
    }

   
    var MyFunc = function(){
     
    }

    
    array[item++] = Assert.expectEq(  "MyFunc.prototype.toString()",       "[object Object]",  MyFunc.prototype.toString() );
    array[item++] = Assert.expectEq(  "typeof MyFunc.prototype",           "object",           typeof MyFunc.prototype );

    return ( array );
}
