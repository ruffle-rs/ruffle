/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "instanceof-003";
//     var VERSION = "ECMA_2";
        
//     var TITLE   = "instanceof operator";
    var BUGNUMBER ="http://bugzilla.mozilla.org/show_bug.cgi?id=7635";

    
    var testcases = getTestCases();
    
function Foo() {};
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var theproto = {};
    Foo.prototype = theproto;

    array[item++] = Assert.expectEq(
        "function Foo() = {}; theproto = {}; Foo.prototype = theproto; " +
            "theproto instanceof Foo",
        false,
        theproto instanceof Foo );


    var o = {};

   /* Assert.expectEq(
        "o = {}; o instanceof o",
        false,
        o instanceof o );*/
    var thisError="no error";
    try{
        o instanceof o;
    }catch(e:Error){
       thisError = e.toString();
    }finally{
        array[item++] = Assert.expectEq(  "o = {}; o instanceof o","TypeError: Error #1040",Utils.typeError(thisError));
    }
    
    return (array);
}
