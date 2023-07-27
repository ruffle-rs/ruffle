/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "10.1.4-1";
//     var VERSION = "ECMA_1";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var myObject2 = new Object();

 

        var MYOBJECT = new MyObject();
        var INPUT = 2;
        myObject2.description += ( INPUT +"" );
 

        with ( MYOBJECT ) {
            eval = function(x){ return(Math.pow(Number(x),3)); }
            // as4 test conversion hack
            myResult = new Object();
            myResult.actual = eval( INPUT );
            myResult.expect = Math.pow(INPUT,3);
            Assert.expectEq( "with MyObject, eval should cube INPUT: ", myResult.expect, myResult.actual );
        }

 

    return ( array );
}

function MyObject() {
    //this.eval = new Function( "x", "return(Math.pow(Number(x),2))" );
    this.eval = function(x){return(Math.pow(Number(x),2));}
}
