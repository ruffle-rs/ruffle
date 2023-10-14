/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11.1.1";
//     var VERSION = "ECMA_1";


    var testcases = getTestCases();
    
function getTestCases(){
    var array = new Array();
    var item = 0;

    var GLOBAL_OBJECT = this.toString();

    // this in global code should return the global object.

    array[item++] = Assert.expectEq(   
                                        "Global Code: this.toString()",
                                        GLOBAL_OBJECT,
                                        this.toString() );

    // this in anonymous code called as a function should return the global object.

    // will work in spidermonkey but will fail in FP7, no compiler error
    var MYFUNC = function(){return this.toString()}
    array[item++] = Assert.expectEq(   
                                        "Anonymous Code: var MYFUNC = new Function('return this.toString()'); MYFUNC()",
                                        GLOBAL_OBJECT,
                                        MYFUNC() );

    // thisin anonymous code called as a function should return that function's activation object
    var MYFUNC = function(){return this.toString();}

    array[item++] = Assert.expectEq(   
                                        "Anonymous Code: var MYFUNC = function(){return this.toString;}",
                                        GLOBAL_OBJECT,
                                        (MYFUNC()).toString() );

    // this in anonymous code called as a constructor should return the object
    var MYFUNC = function(){this.THIS = this}
    array[item++] = Assert.expectEq(   
                                        "Anonymous Code: var MYFUNC = new Function('this.THIS = this'); ((new MYFUNC()).THIS).toString()",
                                        "[object Object]",
                                        ((new MYFUNC()).THIS).toString() );

    var MYFUNC = function(){this.THIS = this}
    var FUN1 = new MYFUNC();
    array[item++] = Assert.expectEq(   
                                        "Anonymous Code: var MYFUNC = new Function('this.THIS = this'); var FUN1 = new MYFUNC(); FUN1.THIS == FUN1",
                                        true,
                                        FUN1.THIS == FUN1 );

    // this in function code called as a function should return the global object.
    array[item++] = Assert.expectEq(   
                                        "Function Code:  ReturnThis()",
                                        GLOBAL_OBJECT,
                                        ReturnThis() );

    //  this in function code called as a contructor should return the object.
    var MYOBJECT = new ReturnThis();
    array[item++] = Assert.expectEq(   
                                        "var MYOBJECT = new ReturnThis(); MYOBJECT.toString()",
                                        "[object Object]",
                                        MYOBJECT.toString() );
    return array;
}


function ReturnThis() {
    return this.toString();
}
