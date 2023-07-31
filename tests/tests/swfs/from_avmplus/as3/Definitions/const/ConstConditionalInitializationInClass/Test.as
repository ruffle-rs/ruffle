/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "conditional initialization inside class constructor";       // Provide ECMA section title or a description
var BUGNUMBER = "";

class MagicBall
{
    const num1:Number;
    
    function MagicBall(cond:Boolean)
    {
        num1 = (cond)? 6 : -8;
    }
    
    function getNumber():Number
    {
        return num1;
    }
}


Assert.expectEq("Conditional initiailization of const inside a class constructor", -8, new MagicBall(false).getNumber());

