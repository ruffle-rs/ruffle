/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "10.1.4-2";
//     var VERSION = "ECMA_1";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    //array[item++] = Assert.expectEq( "with MyObject, eval should return square of ", "", "" );

    
        var MYOBJECT = new MyObject();
        var INPUT = 2;

        var myResult = new Object();

        myResult.description += "( "+INPUT +" )" ;

        with ( this ) {
            with ( MYOBJECT ) {
                myResult.actual = eval( INPUT );
                myResult.expect = Math.pow(INPUT,2);
                Assert.expectEq( "with MyObject, eval should return square of ",  myResult.expect,  myResult.actual );

            }
        }
    

    return ( array );
}

function MyObject() {
    //this.eval = new Function( "x", "return(Math.pow(Number(x),2))" );
    this.eval = function(x){ return(Math.pow(Number(x),2));}
}
