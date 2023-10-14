/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.3.5-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Properties of Function Instances";


    var testcases = getTestCases();

function getTestCases() {
    var array:Array= new Array();
    var item:Number= 0;
        
    var thisError:String="no error";
    try{
        var MyObject = new Function( 'a', 'b', 'c', 'this.a = a; this.b = b; this.c = c; this.value = a+b+c; this.valueOf = new Function( "return this.value" )' );

    }catch(e:Error){
        thisError=(e.toString()).substring(0,22);
    }finally{
        array[item++] = Assert.expectEq( "Function('function body') not supported","EvalError: Error #1066",thisError);
    }

  

    var MyObject = function() {
         this.a = a;
         this.b = b;
         this.c = c;
         this.value = a+b+c;
         this.valueOf = function(){ return this.value}
    }

    array[item++] = Assert.expectEq(  "MyObject.length",                       0,          MyObject.length );
    array[item++] = Assert.expectEq(  "typeof MyObject.prototype",             "object",   typeof MyObject.prototype );
    array[item++] = Assert.expectEq(  "typeof MyObject.prototype.constructor", "function", typeof MyObject.prototype.constructor );
    //not supported anymore
    //array[item++] = Assert.expectEq(  "MyObject.arguments",null,       MyObject.arguments );

    return ( array );
}
