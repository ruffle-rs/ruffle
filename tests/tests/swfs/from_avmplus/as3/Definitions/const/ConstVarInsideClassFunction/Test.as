/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
 
// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "local const inside a class function";       // Provide ECMA section title or a description
var BUGNUMBER = "";


class myClass {
    function constMultiArgs(arg1, arg2, arg3) {
        const localVar = arg1 + arg2 + arg3;
        return localVar;
    }
}

var myVar:Number = -20;

var myObj = new myClass();

Assert.expectEq( "Calling a class function which declares a local const variable", 20 , myObj.constMultiArgs( 10, myVar, 30) );

