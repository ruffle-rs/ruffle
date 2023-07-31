/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {
    const num1:Number = 1;
    const num2:Number = 3;

    public function getNumber(num1:Number, num2:Number)
    {
        return num1 + num2;
    }
}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "const & class function arg with same name";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var obj:Test = new Test();
Assert.expectEq("const and class function arg with same name.  should return the sum of function arg", 2, obj.getNumber(1, 1));

