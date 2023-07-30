/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.3.4.2";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Function.prototype.toString()";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;


    function MyObject( value ) {
        this.value = function() {return this.value;}
        this.toString = function() {return this.value+"";}
    }
    
    var myvar = new MyObject( true );
    myvar.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq(   "myvar = new MyObject( true );  myvar.toString()",'[object Object]',myvar.toString());
        
    myvar = function() {};

    array[item++] = Assert.expectEq(   "myvar = function() {};  myvar.toString()",
                                            "function Function() {}",
                                            myvar.toString());
    
                                 
    array[item++] = Assert.expectEq(   "Function.prototype.toString()",
                                            "function Function() {}",
                                            Function.prototype.toString());
    myvar = Function();
 
    array[item++] = Assert.expectEq(   "myvar = Function();  myvar.toString()","function Function() {}",myvar.toString());

    return ( array );
}
